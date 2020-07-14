extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;

use crate::proc_macro::TokenStream;
use inflections::case::to_snake_case;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::convert::From;
use syn::spanned::Spanned;
use syn::{AttributeArgs, Data, DeriveInput, Error as SynError, Lit, Meta, NestedMeta};

const CEF_ALLOWED_HEADERS: &[&str] = &[
    "Version",
    "DeviceVendor",
    "DeviceProduct",
    "DeviceVersion",
    "DeviceEventClassID",
    "Name",
    "Severity",
];

lazy_static! {
    static ref CEF_ALLOWED_HEADERS_JOINED: String = CEF_ALLOWED_HEADERS.join(",");
    static ref CEF_INVALID_HEADER: String = ["header name should be one of the following:", CEF_ALLOWED_HEADERS_JOINED.as_str()].join(" ");

    static ref CEF_FIXED_HEADERS_USAGE: String = "'cef_fixed_headers' macro expects fixed headers provided in the following syntax: #[cef_fixed_headers(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_FIXED_HEADERS_REDUNDANT: String = ["'cef_fixed_headers' specifies no fixed header values. Remove it or provide values.", CEF_FIXED_HEADERS_USAGE.as_str()].join(" ");
    static ref CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING: String = "'cef_fixed_headers' fixed header values must be strings.".to_owned();
    static ref CEF_FIXED_HEADERS_APPLICATION: String = "'cef_fixed_headers' attribute only applies to Structs or Enums.".to_owned();

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
        return TokenStream::from(
            SynError::new(Span::call_site(), CEF_FIXED_HEADERS_REDUNDANT.to_owned())
                .to_compile_error(),
        );
    }

    let attrs = parse_macro_input!(attr_tokens as AttributeArgs);

    if attrs.len() == 0 {
        return TokenStream::from(
            SynError::new(Span::call_site(), CEF_FIXED_HEADERS_REDUNDANT.to_owned())
                .to_compile_error(),
        );
    }

    // Only applies to items
    let item = parse_macro_input!(item_tokens as DeriveInput);
    match item.data {
        Data::Struct(_) | Data::Enum(_) => {}
        _ => {
            return TokenStream::from(
                SynError::new(Span::call_site(), CEF_FIXED_HEADERS_APPLICATION.to_owned())
                    .to_compile_error(),
            )
        }
    };

    // type name
    let item_name = &item.ident;

    // generics
    let item_generics = &item.generics;
    let (item_impl_generics, item_ty_generics, item_where_clause) = item_generics.split_for_impl();

    let mut trait_impls: Vec<TokenStream2> = vec![];

    for attr in attrs {
        match attr {
            NestedMeta::Meta(meta) => match meta {
                Meta::NameValue(nv) => {
                    let header_name = match nv.path.get_ident() {
                        Some(h) => h,
                        None => {
                            return TokenStream::from(
                                SynError::new(nv.path.span(), CEF_FIXED_HEADERS_USAGE.to_owned())
                                    .to_compile_error(),
                            )
                        }
                    };

                    if !is_valid_header(header_name.to_string().as_str()) {
                        return TokenStream::from(
                            SynError::new(nv.path.span(), CEF_INVALID_HEADER.to_owned())
                                .to_compile_error(),
                        );
                    }

                    let fixed_value = match &nv.lit {
                        Lit::Str(litstr) => litstr,
                        _ => {
                            return TokenStream::from(
                                SynError::new(
                                    nv.lit.span(),
                                    CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING.to_owned(),
                                )
                                .to_compile_error(),
                            )
                        }
                    };

                    let trait_name = format_ident!("CefHeader{}", header_name);
                    let method_name = format_ident!(
                        "cef_header_{}",
                        to_snake_case(header_name.to_string().as_str())
                    );

                    trait_impls.push(quote! {
                        impl #item_impl_generics rust_cef::#trait_name for #item_name #item_ty_generics #item_where_clause {
                            fn #method_name(&self) -> rust_cef::CefResult {
                                Ok(#fixed_value.to_owned())
                            }
                        }
                    });
                }
                _ => {
                    return TokenStream::from(
                        SynError::new(meta.span(), CEF_FIXED_HEADERS_USAGE.to_owned())
                            .to_compile_error(),
                    )
                }
            },
            _ => {
                return TokenStream::from(
                    SynError::new(attr.span(), CEF_FIXED_HEADERS_USAGE.to_owned())
                        .to_compile_error(),
                )
            }
        }
    }

    TokenStream::from(quote! {
        #item

        #(#trait_impls)*
    })
}

fn is_valid_header(id: &str) -> bool {
    for header in CEF_ALLOWED_HEADERS.iter() {
        if id == *header {
            return true;
        }
    }

    false
}
