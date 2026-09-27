use std::sync::atomic::{AtomicBool, Ordering};

use rustc_const_eval::const_eval::eval_to_const_value_raw_provider;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_metadata::creader::CStore;
use rustc_middle::middle::exported_symbols::ExportedSymbol;
use rustc_middle::util::Providers;
use rustc_middle::{
    mir::{
        ConstValue,
        interpret::{AllocInit, Allocation, GlobalId, Pointer, Scalar, alloc_range},
    },
    ty::{PseudoCanonicalInput, TyCtxt, TypeVisitableExt},
};
use rustc_span::{DUMMY_SP, Symbol};

/// Runs inside the compiler that expands `#[register]`: rustc, clippy-driver or any other
/// rustc driver. Nothing happens where no compiler is running, such as rust-analyzer's
/// proc-macro server.
pub(crate) fn install() {
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    rustc_middle::ty::tls::with_opt(|tcx| {
        let Some(tcx) = tcx else { return };
        if INSTALLED.swap(true, Ordering::Relaxed) {
            return;
        }
        // rustc has no hook for this outside a custom driver. Queries read their provider from
        // this table each time they run, so replacing the entry here takes effect for every
        // constant evaluated after macro expansion.
        let providers = &raw const tcx.query_system.local_providers as *mut Providers;
        unsafe { (*providers).queries.eval_to_const_value_raw = evaluate_records };
        // Load every dependency, including ones the code never names, so their records are seen.
        let mut cstore = CStore::from_tcx_mut(tcx);
        for (name, _) in tcx.sess.opts.externs.iter() {
            cstore.process_path_extern(tcx, Symbol::intern(name), DUMMY_SP);
        }
    });
}

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
        return eval_to_const_value_raw_provider(tcx, input);
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
    Ok(ConstValue::Slice {
        alloc_id,
        meta: records.len() as u64,
    })
}

fn registrations(tcx: TyCtxt<'_>) -> Vec<DefId> {
    let mut records: Vec<_> = tcx
        .hir_body_owners()
        .map(|id| id.to_def_id())
        .filter(|&id| registered(tcx, id))
        .collect();
    for &krate in tcx.crates(()) {
        for (symbol, _) in tcx.exported_non_generic_symbols(krate) {
            if let ExportedSymbol::NonGeneric(id) = *symbol
                && registered(tcx, id)
            {
                records.push(id);
            }
        }
    }
    records
}
