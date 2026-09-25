//! Typed registrations collected by Staticizer during Rust constant evaluation.

pub use staticizer_macros::register;

/// Registered values of `T` from the compiling crate and its imported dependencies.
pub struct Records<T: 'static>(std::marker::PhantomData<T>);

impl<T: 'static> Records<T> {
    /// The Staticizer compiler driver supplies this slice for the compiling crate.
    pub const ITEMS: &'static [&'static T] = {
        // Keep evaluation dependent on T until the compiler driver can supply its records.
        let _ = size_of::<T>();
        panic!("build this application with the Staticizer compiler driver")
    };
}
