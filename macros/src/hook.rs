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
/// module path of each impl.
fn evaluate_records<'tcx>(
    tcx: TyCtxt<'tcx>,
    input: PseudoCanonicalInput<'tcx, GlobalId<'tcx>>,
) -> rustc_middle::mir::interpret::EvalToConstValueResult<'tcx> {
    let records = input.value.instance.def_id();
    if tcx.crate_name(records.krate).as_str() != "staticizer"
        || !tcx
            .opt_item_name(records)
            .is_some_and(|name| name.as_str() == "ITEMS")
        || input.value.instance.args.has_non_region_param()
    {
        return eval_to_const_value_raw_provider(tcx, input);
    }
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
            let implemented = tcx
                .impl_trait_ref(id)
                .instantiate_identity()
                .skip_normalization();
            if implemented.args.type_at(1) != expected {
                return None;
            }
            let args = tcx.mk_args(&[implemented.self_ty().into(), expected.into(), app.into()]);
            // Skip an impl for another `A`, such as a plain `Record<T>` when reading
            // `Records<T, App>`; when the same type has impls for several `A`, count only the
            // one this item resolves to.
            let instance = Instance::try_resolve(tcx, typing_env, item, args).ok()??;
            if tcx.parent(instance.def_id()) != id {
                return None;
            }
            let path = tcx.def_path(id).to_string_no_crate_verbose();
            Some(
                tcx.const_eval_instance(typing_env, instance, DUMMY_SP)
                    .map(|value| {
                        (
                            format!("{}{path}", tcx.crate_name(id.krate)),
                            value.try_to_scalar().unwrap(),
                        )
                    }),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    items.sort_by(|left, right| left.0.cmp(&right.0));

    let size = tcx.data_layout.pointer_size();
    let mut allocation = Allocation::new(
        size * items.len() as u64,
        tcx.data_layout.pointer_align().abi,
        AllocInit::Uninit,
        (),
    );
    for (index, (_, pointer)) in items.iter().enumerate() {
        allocation
            .write_scalar(&tcx, alloc_range(size * index as u64, size), *pointer)
            .unwrap();
    }
    allocation.mutability = rustc_ast::Mutability::Not;
    let alloc_id = tcx.reserve_and_set_memory_alloc(tcx.mk_const_alloc(allocation));
    Ok(ConstValue::Slice {
        alloc_id,
        meta: items.len() as u64,
    })
}
