#![feature(rustc_private)]

extern crate rustc_codegen_llvm;
extern crate rustc_codegen_ssa;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;

use rustc_codegen_ssa::{CompiledModules, CrateInfo, TargetConfig, traits::CodegenBackend};
use rustc_const_eval::const_eval::eval_to_const_value_raw_provider;
use rustc_hir::{Mutability, def::DefKind, def_id::DefId};
use rustc_metadata::EncodedMetadata;
use rustc_middle::{
    dep_graph::WorkProductMap,
    middle::exported_symbols::ExportedSymbol,
    mir::{
        ConstValue,
        interpret::{AllocInit, Allocation, EvalToConstValueResult, GlobalId, Pointer, Scalar, alloc_range},
    },
    ty::{PseudoCanonicalInput, TyCtxt, TypeVisitableExt},
    util::Providers,
};
use rustc_session::{CodegenBackendInit, EarlySession, IncrCompSession, Session, config::OutputFilenames};
use std::any::Any;

/// rustc's LLVM backend with the query that supplies `Records::ITEMS` replaced. rustc loads it
/// through `-Zcodegen-backend`, so it works under any driver, including `clippy-driver`.
struct Staticizer(Box<dyn CodegenBackend>);

#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(Staticizer(rustc_codegen_llvm::LlvmCodegenBackend::new()))
}

impl CodegenBackend for Staticizer {
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn init(&mut self, sess: &EarlySession) -> CodegenBackendInit {
        self.0.init(sess)
    }

    fn target_config(&self, sess: &EarlySession) -> TargetConfig {
        self.0.target_config(sess)
    }

    fn provide(&self, providers: &mut Providers) {
        providers.queries.eval_to_const_value_raw = evaluate_records;
    }

    fn target_cpu(&self, sess: &Session) -> String {
        self.0.target_cpu(sess)
    }

    fn codegen_crate<'tcx>(&self, tcx: TyCtxt<'tcx>) -> Box<dyn Any> {
        self.0.codegen_crate(tcx)
    }

    fn join_codegen(
        &self,
        codegen: Box<dyn Any>,
        sess: &Session,
        incremental: Option<&IncrCompSession>,
        outputs: &OutputFilenames,
        info: &CrateInfo,
    ) -> (CompiledModules, WorkProductMap) {
        self.0.join_codegen(codegen, sess, incremental, outputs, info)
    }

    fn link(
        &self,
        sess: &Session,
        modules: CompiledModules,
        info: CrateInfo,
        metadata: EncodedMetadata,
        outputs: &OutputFilenames,
    ) {
        self.0.link(sess, modules, info, metadata, outputs)
    }
}

fn evaluate_records<'tcx>(
    tcx: TyCtxt<'tcx>,
    input: PseudoCanonicalInput<'tcx, GlobalId<'tcx>>,
) -> EvalToConstValueResult<'tcx> {
    let instance = input.value.instance;
    if tcx.crate_name(instance.def_id().krate).as_str() != "staticizer"
        || tcx.opt_item_name(instance.def_id()).is_none_or(|name| name.as_str() != "ITEMS")
        || instance.args.has_non_region_param()
    {
        return eval_to_const_value_raw_provider(tcx, input);
    }
    let expected = instance.args.type_at(0);
    let mut records: Vec<_> = registrations(tcx)
        .filter(|&id| tcx.type_of(id).instantiate_identity().skip_normalization() == expected)
        .collect();
    records.sort_by_cached_key(|&id| tcx.codegen_fn_attrs(id).symbol_name.unwrap().to_string());
    let size = tcx.data_layout.pointer_size();
    let align = tcx.data_layout.pointer_align().abi;
    let mut allocation = Allocation::new(size * records.len() as u64, align, AllocInit::Uninit, ());
    for (index, &id) in records.iter().enumerate() {
        let pointer = Pointer::from(tcx.reserve_and_set_static_alloc(id));
        let range = alloc_range(size * index as u64, size);
        allocation.write_scalar(&tcx, range, Scalar::from_pointer(pointer, &tcx)).unwrap();
    }
    allocation.mutability = Mutability::Not;
    let alloc_id = tcx.reserve_and_set_memory_alloc(tcx.mk_const_alloc(allocation));
    Ok(ConstValue::Slice { alloc_id, meta: records.len() as u64 })
}

/// Registered statics of the compiling crate and of every loaded crate.
fn registrations(tcx: TyCtxt<'_>) -> impl Iterator<Item = DefId> {
    let exported = tcx.crates(()).iter().flat_map(move |&krate| tcx.exported_non_generic_symbols(krate));
    tcx.hir_body_owners()
        .map(|id| id.to_def_id())
        .chain(exported.filter_map(|&(symbol, _)| match symbol {
            ExportedSymbol::NonGeneric(id) => Some(id),
            _ => None,
        }))
        .filter(move |&id| {
            matches!(tcx.def_kind(id), DefKind::Static { .. })
                && tcx
                    .codegen_fn_attrs(id)
                    .symbol_name
                    .is_some_and(|name| name.as_str().starts_with("__staticizer_record_"))
        })
}
