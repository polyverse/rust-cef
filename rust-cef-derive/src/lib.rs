extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;

use crate::proc_macro::{TokenStream};
use syn::{DeriveInput, Attribute, AttributeArgs, Meta, Error as SynError, Lit, MetaNameValue, NestedMeta};
use syn::spanned::Spanned;
use std::convert::{From};
use inflections::case::to_snake_case;

const CEF_ALLOWED_HEADERS: &[&str] = &[
    "Version",
    "DeviceVendor",
    "DeviceProduct",
    "DeviceVersion",
    "DeviceEventClassID",
    "Name",
    "Severity"
];

lazy_static! {
    static ref CEF_ALLOWED_HEADERS_JOINED: String = CEF_ALLOWED_HEADERS.join(",");
    static ref CEF_INVALID_HEADER: String = ["header name should be one of the following:", CEF_ALLOWED_HEADERS_JOINED.as_str()].join(" ");

    static ref CEF_FIXED_HEADERS_USAGE: String = "'cef_fixed_headers' macro expects fixed headers provided in the following syntax: #[cef_fixed_headers(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_FIXED_HEADERS_REDUNDANT: String = ["'cef_fixed_headers' specifies no fixed header values. Remove it or provide values.", CEF_FIXED_HEADERS_USAGE.as_str()].join(" ");
    static ref CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING: String = ["'cef_fixed_headers' fixed header values must be strings.", CEF_FIXED_HEADERS_USAGE.as_str()].join(" ");

    static ref CEF_HEADER_USAGE: String = "'cef_header' macro expects header name provided in the following syntax: #[cef_header(header_name)] ".to_owned();
    static ref CEF_HEADER_REDUNDANT: String = ["'cef_header' macro is redundant since no header name is provided.", CEF_HEADER_USAGE.as_str()].join(" ");
    static ref CEF_HEADER_ONLY_ONE: String = ["'cef_header' macro has more than one header name provided. A field can only be converted into one header.", CEF_HEADER_USAGE.as_str()].join(" ");

}

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

#[proc_macro_attribute]
pub fn cef_fixed_headers(attr_tokens: TokenStream, item_tokens: TokenStream) -> TokenStream {
    if attr_tokens.is_empty() {
        return TokenStreams::from(SynError::new(attr_tokens.span(), CEF_FIXED_HEADERS_REDUNDANT.to_owned()));
    }

    let attrs = parse_macro_input!(attr_tokens as AttributeArgs);
    println!("{:#?}", attrs);

    item_tokens
}

fn get_fixed_header_values(attr: Attribute, impl_generics: &syn::ImplGenerics, name: &syn::Ident, ty_generics: &syn::TypeGenerics, where_clause: &syn::WhereClause) -> (proc_macro2::TokenStream, bool) {
    match attr.parse_meta() {
        Ok(meta) => {
            match meta {
                Meta::NameValue(nv) => {
                    get_fixed_header_value(nv, impl_generics, name, ty_generics, where_clause)
                },
                Meta::List(metalist) => {
                    if metalist.nested.is_empty() {
                        return (SynError::new(metalist.span(), CEF_FIXED_HEADERS_REDUNDANT.as_str()).to_compile_error(), true)
                    }

                    let mut fixed_header_traits: Vec<proc_macro2::TokenStream> = vec![];
                    for nested_meta in metalist.nested {
                        match nested_meta {
                            NestedMeta::Meta(m) => match m {
                                Meta::NameValue(nv) => {
                                    let (fixed_header_trait, compile_error) = get_fixed_header_value(nv, impl_generics, name, ty_generics, where_clause);
                                    if compile_error {
                                        return (fixed_header_trait, compile_error);
                                    }

                                    fixed_header_traits.push(fixed_header_trait);
                                },
                                _ => return (SynError::new(m.span(), CEF_FIXED_HEADERS_USAGE.as_str()).to_compile_error(), true),
                            },
                            NestedMeta::Lit(nestedlist) => return (SynError::new(nestedlist.span(), CEF_FIXED_HEADERS_USAGE.as_str()).to_compile_error(), true),
                        }
                    }

                    (quote! {
                        #(#fixed_header_traits)*
                    }, false)
                },
                _ => return (SynError::new(meta.span(), CEF_FIXED_HEADERS_USAGE.as_str()).to_compile_error(), true),
            }
        },
        Err(e) => return (e.to_compile_error(), true)
    }
}

fn get_fixed_header_value(nv: MetaNameValue, impl_generics: &syn::ImplGenerics, name: &syn::Ident, ty_generics: &syn::TypeGenerics, where_clause: &syn::WhereClause) -> (proc_macro2::TokenStream, bool) {
    let maybe_pathname = nv.path.get_ident();
    if maybe_pathname.is_none() {
        return (SynError::new(nv.span(), CEF_INVALID_HEADER.as_str()).to_compile_error(), true)
    }
    let pathname = maybe_pathname.unwrap().to_string();
    if !is_valid_header(&pathname) {
        return (SynError::new(nv.span(), CEF_INVALID_HEADER.as_str()).to_compile_error(), true)
    }

    let value = match nv.lit {
        Lit::Str(s) => s,
        _ => return (SynError::new(nv.span(), CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING.as_str()).to_compile_error(), true),
    };

    let trait_name = format_ident!("CefHeader{}", pathname);
    let method_name = format_ident!("cef_header_{}", to_snake_case(pathname.to_string().as_str()));

    (quote! {
        impl #impl_generics rust_cef::#trait_name for #name #ty_generics #where_clause {
            fn #method_name() -> rust_cef::CefResult {
                Ok(#value)
            }
        }
    }, false)
}


fn is_valid_header(id: &str) -> bool {
    for header in CEF_ALLOWED_HEADERS.iter() {
        if id == *header {
            return true;
        }
    }

    false
}
