use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStatic, LitStr, parse_macro_input};

/// Makes immutable metadata discoverable by the compiler driver.
#[proc_macro_attribute]
pub fn register(group: TokenStream, item: TokenStream) -> TokenStream {
    let group = parse_macro_input!(group as LitStr);
    let item = parse_macro_input!(item as ItemStatic);
    if matches!(item.mutability, syn::StaticMutability::Mut(_)) {
        return syn::Error::new_spanned(item, "registered metadata must be immutable")
            .into_compile_error()
            .into();
    }
    let name = &item.ident;
    quote! {
        #[used]
        #[unsafe(export_name = concat!("__staticizer_record_", #group, "::", module_path!(), "::", stringify!(#name)))]
        #item
    }.into()
}
