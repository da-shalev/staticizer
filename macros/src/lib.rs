#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_span;

mod hook;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStatic, parse::Nothing, parse_macro_input};

/// Makes immutable metadata discoverable through `Records::<T>::ITEMS`, and installs the hook
/// that supplies those records in the compiler expanding it.
#[proc_macro_attribute]
pub fn register(args: TokenStream, item: TokenStream) -> TokenStream {
    hook::install();
    parse_macro_input!(args as Nothing);
    let item = parse_macro_input!(item as ItemStatic);
    if matches!(item.mutability, syn::StaticMutability::Mut(_)) {
        return syn::Error::new_spanned(item, "registered metadata must be immutable")
            .into_compile_error()
            .into();
    }
    let name = &item.ident;
    quote! {
        #[used]
        #[unsafe(export_name = concat!("__staticizer_record_", module_path!(), "::", stringify!(#name)))]
        #item
    }.into()
}
