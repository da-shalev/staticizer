use rustc_hir::attrs::{InlineAttr, Linkage};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;
use rustc_middle::middle::exported_symbols::{
    ExportedSymbol, SymbolExportInfo, SymbolExportKind, SymbolExportLevel,
};
use rustc_middle::mir::{
    self, ConstValue,
    interpret::{Pointer, Scalar},
};
use rustc_middle::mono::MonoItem;
use rustc_middle::query::LocalCrate;
use rustc_middle::ty::{self, Instance, SymbolName, TyCtxt};
use rustc_span::DUMMY_SP;
use std::sync::OnceLock;

type Mir = for<'tcx> fn(TyCtxt<'tcx>, LocalDefId) -> &'tcx mir::Body<'tcx>;
type Attrs = for<'tcx> fn(TyCtxt<'tcx>, DefId) -> CodegenFnAttrs;
type Symbol = for<'tcx> fn(TyCtxt<'tcx>, Instance<'tcx>) -> SymbolName<'tcx>;
type Exports =
    for<'tcx> fn(TyCtxt<'tcx>, LocalCrate) -> &'tcx [(ExportedSymbol<'tcx>, SymbolExportInfo)];

static ORIGINAL_MIR: OnceLock<Mir> = OnceLock::new();
static ORIGINAL_ATTRIBUTES: OnceLock<Attrs> = OnceLock::new();
static ORIGINAL_SYMBOL: OnceLock<Symbol> = OnceLock::new();
static ORIGINAL_EXPORTS: OnceLock<Exports> = OnceLock::new();

pub fn install(providers: &mut rustc_middle::util::Providers) {
    ORIGINAL_MIR.set(providers.queries.optimized_mir).unwrap();
    providers.queries.optimized_mir = root_outputs;
    ORIGINAL_ATTRIBUTES
        .set(providers.extern_queries.codegen_fn_attrs)
        .unwrap();
    providers.extern_queries.codegen_fn_attrs = output_attributes;
    ORIGINAL_SYMBOL.set(providers.queries.symbol_name).unwrap();
    providers.queries.symbol_name = output_symbol;
    ORIGINAL_EXPORTS
        .set(providers.queries.exported_generic_symbols)
        .unwrap();
    providers.queries.exported_generic_symbols = export_outputs;
}

fn is_output(tcx: TyCtxt<'_>, id: DefId) -> bool {
    tcx.crate_name(id.krate).as_str() == "staticizer"
        && tcx
            .opt_item_name(id)
            .is_some_and(|name| name.as_str() == "output")
}

fn output_attributes(tcx: TyCtxt<'_>, id: DefId) -> CodegenFnAttrs {
    let mut attrs = ORIGINAL_ATTRIBUTES.get().unwrap()(tcx, id);
    if tcx.entry_fn(()).is_some() && is_output(tcx, id) {
        attrs.linkage = Some(Linkage::External);
        attrs.inline = InlineAttr::None;
    }
    attrs
}

fn outputs<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<Instance<'tcx>> {
    let mut instances = Vec::new();
    for &krate in tcx.crates(()) {
        for &(symbol, _) in tcx.exported_generic_symbols(krate) {
            if let ExportedSymbol::Generic(id, args) = symbol
                && is_output(tcx, id)
            {
                instances.push(Instance::new_raw(id, args));
            }
        }
    }
    instances.sort_by_cached_key(|instance| tcx.symbol_name(*instance));
    instances.dedup();
    instances
}

fn root_outputs<'tcx>(tcx: TyCtxt<'tcx>, id: LocalDefId) -> &'tcx mir::Body<'tcx> {
    let original = ORIGINAL_MIR.get().unwrap()(tcx, id);
    if !tcx
        .entry_fn(())
        .is_some_and(|(entry, _)| entry == id.to_def_id())
    {
        return original;
    }
    let mut body = original.clone();
    // These unused function pointers make the compiler instantiate every final output getter.
    for instance in outputs(tcx) {
        let alloc_id = tcx.reserve_and_set_fn_alloc(instance, 0);
        let scalar =
            Scalar::from_pointer(Pointer::new(alloc_id.into(), rustc_abi::Size::ZERO), &tcx);
        let signature = tcx
            .fn_sig(instance.def_id())
            .instantiate(tcx, instance.args)
            .skip_normalization();
        let ty = ty::Ty::new_fn_ptr(tcx, signature);
        let value = mir::ConstOperand {
            span: DUMMY_SP,
            user_ty: None,
            const_: mir::Const::Val(ConstValue::Scalar(scalar), ty),
        };
        let local = body
            .local_decls
            .push(mir::LocalDecl::new(value.const_.ty(), DUMMY_SP));
        body.basic_blocks.as_mut()[mir::START_BLOCK]
            .statements
            .push(mir::Statement::new(
                mir::SourceInfo::outermost(DUMMY_SP),
                mir::StatementKind::Assign(Box::new((
                    mir::Place::from(local),
                    mir::Rvalue::Use(mir::Operand::Constant(Box::new(value)), mir::WithRetag::No),
                ))),
            ));
    }
    tcx.arena.alloc(body)
}

fn output_symbol<'tcx>(tcx: TyCtxt<'tcx>, instance: Instance<'tcx>) -> SymbolName<'tcx> {
    let id = instance.def_id();
    if is_output(tcx, id) {
        return SymbolName::new(
            tcx,
            &rustc_symbol_mangling::symbol_name_for_instance_in_crate(tcx, instance, id.krate),
        );
    }
    ORIGINAL_SYMBOL.get().unwrap()(tcx, instance)
}

fn export_outputs<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: LocalCrate,
) -> &'tcx [(ExportedSymbol<'tcx>, SymbolExportInfo)] {
    let original = ORIGINAL_EXPORTS.get().unwrap()(tcx, key);
    if !tcx.sess.opts.output_types.should_codegen() {
        return original;
    }
    let mut symbols = original.to_vec();
    let info = SymbolExportInfo {
        level: SymbolExportLevel::Rust,
        kind: SymbolExportKind::Text,
        used: true,
        rustc_std_internal_symbol: false,
    };
    if tcx.entry_fn(()).is_none() {
        // rustc normally excludes weak functions from generic exports. Record their
        // concrete types in metadata so the application can rebuild these outputs.
        for cgu in tcx.collect_and_partition_mono_items(()).codegen_units {
            for item in cgu.items().keys() {
                if let MonoItem::Fn(instance) = *item
                    && is_output(tcx, instance.def_id())
                {
                    symbols.push((
                        ExportedSymbol::Generic(instance.def_id(), instance.args),
                        info,
                    ));
                }
            }
        }
    } else {
        // LTO must preserve the final definition for callers compiled in dependencies.
        // NoDefId preserves our canonical name; Generic would remangle it for this crate.
        symbols.extend(
            outputs(tcx)
                .into_iter()
                .map(|instance| (ExportedSymbol::NoDefId(tcx.symbol_name(instance)), info)),
        );
    }
    // Codegen-unit maps have no stable iteration order; metadata must have one.
    symbols.sort_by_cached_key(|(symbol, _)| symbol.symbol_name_for_local_instance(tcx));
    symbols.dedup_by_key(|(symbol, _)| *symbol);
    tcx.arena.alloc_from_iter(symbols)
}
