#![feature(linkage)]

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

/// A value computed from the final program's registrations.
pub trait Build: Sized + 'static {
    const VALUE: &'static Self;
}

/// Returns the value built from the final application's registrations.
///
/// The compiler driver discovers concrete calls, including those in dependencies,
/// and constructs their outputs from the final application's registrations.
/// Read the result through this function; a dependency's `Build::VALUE` only sees its own inputs.
#[inline(never)]
#[linkage = "weak"]
pub fn output<T: Build>() -> &'static T {
    T::VALUE
}
