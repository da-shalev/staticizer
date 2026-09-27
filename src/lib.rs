//! Typed registrations collected by Staticizer during Rust constant evaluation.

pub use staticizer_macros::register;

/// Registered values of `T` from the compiling crate and its imported dependencies.
pub struct Records<T: 'static>(std::marker::PhantomData<T>);

impl<T: 'static> Records<T> {
    /// Supplied by the hook `#[staticizer::register]` installs in the compiling crate.
    pub const ITEMS: &'static [&'static T] = {
        // Keep evaluation dependent on T so the hook can supply its records.
        let _ = size_of::<T>();
        panic!("use `#[staticizer::register]` in the crate that evaluates `Records::ITEMS`")
    };
}
