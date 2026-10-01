//! Typed registrations collected by Staticizer during Rust constant evaluation.

pub use staticizer_macros::register;

/// A registered value of `T`. Every `#[register]` impl in the compiling crate and its
/// dependencies is part of `Records::<T, A>::ITEMS`.
pub trait Record<T: 'static, A: 'static = ()> {
    const ITEM: &'static T;
}

/// `ITEM` of every registered `Record<T, A>` impl, in an order that is the same on every build.
///
/// `A` defers evaluation to the crate that names it: generic code keyed by `A` sees the
/// records of whichever crate supplies `A`, and needs no `#[register]` of its own.
pub struct Records<T: 'static, A: 'static = ()>(std::marker::PhantomData<(T, A)>);

impl<T: 'static, A: 'static> Records<T, A> {
    /// Supplied by the hook `#[staticizer::register]` installs in the compiling crate.
    pub const ITEMS: &'static [&'static T] = {
        // Keep evaluation dependent on T and A so the hook can supply its records.
        let _ = (size_of::<T>(), size_of::<A>());
        panic!("use `#[staticizer::register]` in the crate that evaluates `Records::ITEMS`")
    };
}
