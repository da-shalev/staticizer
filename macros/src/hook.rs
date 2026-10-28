use std::sync::atomic::{AtomicBool, Ordering};

use rustc_const_eval::const_eval::eval_to_const_value_raw_provider;
use rustc_metadata::creader::CStore;
use rustc_middle::util::Providers;
use rustc_middle::{
    mir::{
        ConstValue,
        interpret::{AllocInit, Allocation, GlobalId, alloc_range},
    },
    ty::{Instance, PseudoCanonicalInput, TyCtxt, TypeVisitableExt, TypingEnv},
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

/// `Records::<T, A>::ITEMS`: the `ITEM` pointer of every `Record<T, A>` impl, ordered by the
/// stable hash rustc gives each impl, so the order is the same on every build. `Position::OF` is
/// where `M`'s impl sits in that order.
fn evaluate_records<'tcx>(
    tcx: TyCtxt<'tcx>,
    input: PseudoCanonicalInput<'tcx, GlobalId<'tcx>>,
) -> rustc_middle::mir::interpret::EvalToConstValueResult<'tcx> {
    let records = input.value.instance.def_id();
    let supplied = tcx.crate_name(records.krate).as_str() == "staticizer"
        && !input.value.instance.args.has_non_region_param();
    let position = match tcx.opt_item_name(records).as_ref().map(Symbol::as_str) {
        Some("ITEMS") if supplied => false,
        Some("OF") if supplied => true,
        _ => return eval_to_const_value_raw_provider(tcx, input),
    };
    let (expected, app) = (
        input.value.instance.args.type_at(0),
        input.value.instance.args.type_at(1),
    );
    let record = *tcx
        .traits(records.krate)
        .iter()
        .find(|&&id| tcx.item_name(id).as_str() == "Record")
        .unwrap();
    let item = tcx.associated_item_def_ids(record)[0];
    let typing_env = TypingEnv::fully_monomorphized();
    let mut items = tcx
        .all_impls(record)
        .filter_map(|id| {
            // The types being read have their lifetimes erased, so erase the impl's to match.
            let implemented = tcx.erase_and_anonymize_regions(
                tcx.impl_trait_ref(id)
                    .instantiate_identity()
                    .skip_normalization(),
            );
            let args = tcx.mk_args(&[implemented.self_ty().into(), expected.into(), app.into()]);
            // Keep the impl only if this is the impl `Record<T, A>` resolves to for its type:
            // impls for another `T` or `A` resolve elsewhere or nowhere.
            let instance = Instance::try_resolve(tcx, typing_env, item, args).ok()??;
            (tcx.parent(instance.def_id()) == id)
                .then(|| (tcx.def_path_hash(id), implemented.self_ty(), instance))
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|&(hash, ..)| hash);

    if position {
        let marker = input.value.instance.args.type_at(2);
        let Some(index) = items.iter().position(|&(_, ty, _)| ty == marker) else {
            return eval_to_const_value_raw_provider(tcx, input);
        };
        return Ok(ConstValue::from_target_usize(index as u64, &tcx));
    }

    let size = tcx.data_layout.pointer_size();
    let mut allocation = Allocation::new(
        size * items.len() as u64,
        tcx.data_layout.pointer_align().abi,
        AllocInit::Uninit,
        (),
    );
    for (index, &(_, _, instance)) in items.iter().enumerate() {
        let pointer = tcx.const_eval_instance(typing_env, instance, DUMMY_SP)?;
        allocation
            .write_scalar(
                &tcx,
                alloc_range(size * index as u64, size),
                pointer.try_to_scalar().unwrap(),
            )
            .unwrap();
    }
    allocation.mutability = rustc_ast::Mutability::Not;
    let alloc_id = tcx.reserve_and_set_memory_alloc(tcx.mk_const_alloc(allocation));
    Ok(ConstValue::Slice {
        alloc_id,
        meta: items.len() as u64,
    })
}
