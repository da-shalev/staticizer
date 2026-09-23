#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_symbol_mangling;

mod finish;

use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::middle::exported_symbols::ExportedSymbol;
use rustc_middle::{
    mir::{
        ConstValue,
        interpret::{AllocInit, Allocation, GlobalId, Pointer, Scalar, alloc_range},
    },
    ty::{PseudoCanonicalInput, TyCtxt, TypeVisitableExt},
};
use std::sync::OnceLock;

type Eval = for<'tcx> fn(
    TyCtxt<'tcx>,
    PseudoCanonicalInput<'tcx, GlobalId<'tcx>>,
) -> rustc_middle::mir::interpret::EvalToConstValueResult<'tcx>;

static ORIGINAL_EVAL: OnceLock<Eval> = OnceLock::new();

fn registered(tcx: TyCtxt<'_>, id: DefId) -> bool {
    matches!(tcx.def_kind(id), DefKind::Static { .. })
        && tcx
            .codegen_fn_attrs(id)
            .symbol_name
            .is_some_and(|name| name.as_str().starts_with("__staticizer_record_"))
}

fn evaluate_records<'tcx>(
    tcx: TyCtxt<'tcx>,
    input: PseudoCanonicalInput<'tcx, GlobalId<'tcx>>,
) -> rustc_middle::mir::interpret::EvalToConstValueResult<'tcx> {
    if tcx.crate_name(input.value.instance.def_id().krate).as_str() != "staticizer"
        || !tcx
            .opt_item_name(input.value.instance.def_id())
            .is_some_and(|name| name.as_str() == "ITEMS")
        || input.value.instance.args.has_non_region_param()
    {
        return ORIGINAL_EVAL.get().unwrap()(tcx, input);
    }
    let expected = input.value.instance.args.type_at(0);
    let mut records: Vec<_> = registrations(tcx)
        .into_iter()
        .filter(|id| tcx.type_of(*id).instantiate_identity().skip_normalization() == expected)
        .collect();
    records.sort_by(|left, right| {
        let left = tcx.codegen_fn_attrs(*left).symbol_name.unwrap();
        let right = tcx.codegen_fn_attrs(*right).symbol_name.unwrap();
        left.as_str().cmp(right.as_str())
    });
    let size = tcx.data_layout.pointer_size();
    let mut allocation = Allocation::new(
        size * records.len() as u64,
        tcx.data_layout.pointer_align().abi,
        AllocInit::Uninit,
        (),
    );
    for (index, id) in records.iter().enumerate() {
        let pointer = Pointer::new(
            tcx.reserve_and_set_static_alloc(*id).into(),
            rustc_abi::Size::ZERO,
        );
        allocation
            .write_scalar(
                &tcx,
                alloc_range(size * index as u64, size),
                Scalar::from_pointer(pointer, &tcx),
            )
            .unwrap();
    }
    allocation.mutability = rustc_ast::Mutability::Not;
    let alloc_id = tcx.reserve_and_set_memory_alloc(tcx.mk_const_alloc(allocation));
    records_log(tcx, expected, records.len());
    Ok(ConstValue::Slice {
        alloc_id,
        meta: records.len() as u64,
    })
}

struct StaticizerCallbacks;

impl rustc_driver::Callbacks for StaticizerCallbacks {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.override_queries = Some(|_, providers| {
            ORIGINAL_EVAL
                .set(providers.queries.eval_to_const_value_raw)
                .unwrap();
            providers.queries.eval_to_const_value_raw = evaluate_records;
            finish::install(providers);
        });
    }
}

fn records_log(tcx: TyCtxt<'_>, ty: rustc_middle::ty::Ty<'_>, count: usize) {
    if std::env::var_os("STATICIZER_TRACE").is_some() {
        eprintln!(
            "staticizer: {}: {count} records of {ty:?}",
            tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE)
        );
    }
}

fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|arg| {
        std::path::Path::new(arg)
            .file_stem()
            .is_some_and(|name| name == "rustc")
    }) {
        args.remove(1);
    }
    args.push("-Zshare-generics=no".into());
    rustc_driver::run_compiler(&args, &mut StaticizerCallbacks);
}

fn registrations(tcx: TyCtxt<'_>) -> Vec<DefId> {
    let mut records: Vec<_> = tcx
        .hir_body_owners()
        .map(|id| id.to_def_id())
        .filter(|&id| registered(tcx, id))
        .collect();
    for &krate in tcx.crates(()) {
        for (symbol, _) in tcx.exported_non_generic_symbols(krate) {
            if let ExportedSymbol::NonGeneric(id) = *symbol {
                if registered(tcx, id) {
                    records.push(id);
                }
            }
        }
    }
    records
}
