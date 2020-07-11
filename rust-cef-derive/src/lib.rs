extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;

use crate::proc_macro::TokenStream;
use syn::{DeriveInput, AttributeArgs, NestedMeta, Meta, Error as SynError};
use syn::spanned::Spanned;
use std::convert::{From};

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

    static ref CEF_FIXED_HEADERS_USAGE: String = "cef_fixed_headers macro expects fixed headers provided in the following syntax: #[cef_fixed_headers(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_INVALID_HEADER: String = ["header name should be one of the following:", CEF_ALLOWED_HEADERS_JOINED.as_str()].join(" ");
}

#[proc_macro_attribute]
pub fn cef_fixed_headers(attr: TokenStream, item: TokenStream) -> TokenStream {
    // validate attribute
    let nestedmetavec = parse_macro_input!(attr as AttributeArgs);
    for nestedmeta in nestedmetavec {
        let span = nestedmeta.span().to_owned();

        match nestedmeta {
            NestedMeta::Meta(meta) => match meta {
                Meta::NameValue(nv) => {
                    let pathname = nv.path.get_ident();
                    if pathname.is_none() {
                        return TokenStream::from(SynError::new(span, CEF_INVALID_HEADER.as_str()).to_compile_error())
                    }
                    if !valid_header(&pathname.unwrap().to_string()) {
                        return TokenStream::from(SynError::new(span, CEF_INVALID_HEADER.as_str()).to_compile_error())
                    }
                },
                _ => return TokenStream::from(SynError::new(span, CEF_FIXED_HEADERS_USAGE.as_str()).to_compile_error()),
            },
            _ => return TokenStream::from(SynError::new(span, CEF_FIXED_HEADERS_USAGE.as_str()).to_compile_error()),
        }
    }

    item
}

#[proc_macro_derive(ToCef)]
pub fn writable_template_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // attrs
    let attrs = input.attrs;
    for attr in &attrs {
        let _meta = attr.parse_meta().unwrap();
    }

    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::ToCef for #name #ty_generics #where_clause {
            fn to_cef(&self) -> rust_cef::CefResult {
                Ok("".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
}


fn valid_header(id: &str) -> bool {
    for header in CEF_ALLOWED_HEADERS.iter() {
        if id == *header {
            return true;
        }
    }

    false
}
