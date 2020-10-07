/// This crate provides macros to generate boiler-plate CEF-trait code
/// to provide fixed values, inherit values and bubble them up to a larger item
/// or to convert fields/variants into header values.
///
/// It also allows anotating fields to have specific names in CEF extensions.
///
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;

mod cef_extensions_trait;
mod cef_header_traits;
mod helpers;

use crate::proc_macro::TokenStream;
use cef_extensions_trait::implement_extensions_trait;
use cef_header_traits::implement_header_trait;
use std::convert::From;
use syn::DeriveInput;

/// This macro derives the ToCef trait on the annotated item.
/// For now, the ToCef trait itself provides a useful implementation
/// of the trait, so this macro simply implements a blank trait.
///
#[proc_macro_derive(ToCef)]
pub fn derive_to_cef(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // default implementation is great
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::ToCef for #name #ty_generics #where_clause {}
    };

    TokenStream::from(to_cef_impl)
}

#[proc_macro_derive(CefHeaderVersion, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_version(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderVersion", item_tokens)
}

#[proc_macro_derive(CefHeaderDeviceVendor, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_vendor(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderDeviceVendor", item_tokens)
}

#[proc_macro_derive(CefHeaderDeviceProduct, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_product(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderDeviceProduct", item_tokens)
}

#[proc_macro_derive(CefHeaderDeviceVersion, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_version(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderDeviceVersion", item_tokens)
}
#[proc_macro_derive(
    CefHeaderDeviceEventClassID,
    attributes(cef_values, cef_inherit, cef_field)
)]
pub fn derive_cef_header_device_event_class_id(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderDeviceEventClassID", item_tokens)
}
#[proc_macro_derive(CefHeaderName, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_name(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderName", item_tokens)
}
#[proc_macro_derive(CefHeaderSeverity, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_severity(item_tokens: TokenStream) -> TokenStream {
    implement_header_trait("CefHeaderSeverity", item_tokens)
}

#[proc_macro_derive(
    CefExtensions,
    attributes(cef_ext_field, cef_ext_gobble, cef_ext_values)
)]
pub fn derive_cef_extensions(input: TokenStream) -> TokenStream {
    implement_extensions_trait(input)
}
