#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_span;

mod hook;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, parse::Nothing, parse_macro_input};

/// Makes a `Record` impl's `ITEM` part of `Records::ITEMS`, and installs the hook that supplies
/// those records in the compiler expanding it.
#[proc_macro_attribute]
pub fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    hook::install();
    parse_macro_input!(args as Nothing);
    let item = parse_macro_input!(item as ItemImpl);
    // Only the hook reads a record, which the dead code lint cannot see.
    quote! {
        #[allow(dead_code)]
        #item
    }
    .into()
}
