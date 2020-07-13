extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate lazy_static;

use crate::proc_macro::{TokenStream};
use syn::{DeriveInput, Attribute, Meta, Error as SynError, Lit, MetaNameValue, NestedMeta};
use syn::spanned::Spanned;
use std::convert::{From};
use inflector::cases::snakecase;


lazy_static! {
    static ref CEF_ALLOWED_HEADERS_JOINED: String = CEF_ALLOWED_HEADERS.join(",");
    static ref CEF_INVALID_HEADER: String = ["header name should be one of the following:", CEF_ALLOWED_HEADERS_JOINED.as_str()].join(" ");

    static ref CEF_FIXED_HEADERS_USAGE: String = "'cef_fixed_headers' macro expects fixed headers provided in the following syntax: #[cef_fixed_headers(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_FIXED_HEADERS_REDUNDANT: String = ["'cef_fixed_headers' macro is redundant since no fixed header values are provided.", CEF_FIXED_HEADERS_USAGE.as_str()].join(" ");
    static ref CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING: String = ["'cef_fixed_headers' fixed header values must be strings.", CEF_FIXED_HEADERS_USAGE.as_str()].join(" ");

    static ref CEF_HEADER_USAGE: String = "'cef_header' macro expects header name provided in the following syntax: #[cef_header(header_name)] ".to_owned();
    static ref CEF_HEADER_REDUNDANT: String = ["'cef_header' macro is redundant since no header name is provided.", CEF_HEADER_USAGE.as_str()].join(" ");
    static ref CEF_HEADER_ONLY_ONE: String = ["'cef_header' macro has more than one header name provided. A field can only be converted into one header.", CEF_HEADER_USAGE.as_str()].join(" ");

}

#[proc_macro_derive(ToCef, attributes(cef_fixed_headers, cef_header))]
pub fn to_cef_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // span
    let span = input.span();

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    /*
    // headers to satisfy
    let mut unsatisfied_headers = CEF_ALLOWED_HEADERS.to_vec();

    let mut header_traits: Vec<proc_macro2::TokenStream> = vec![];
    let attrs = input.attrs;
    for attr in attrs {
        if attr.path.is_ident("cef_fixed_headers") {
            let (header_trait, compile_error) = get_fixed_header_values(attr, &mut unsatisfied_headers);
            if compile_error {
                return TokenStream::from(header_value);
            }
        } else if attr.path.is_ident("cef_header") {
            let (header_trait, compile_error) = get_dynamic_header_values(attr, &mut unsatisfied_headers);
            if compile_error {
                return TokenStream::from(header_value);
            }
        }
        header_traits.push(header_value);
    }

    if unsatisfied_headers.len() > 0 {
        return TokenStream::from(SynError::new(span, format!("Unable to derive ToCef since mandatory header values not provided (either fixed or dynamic): {}", unsatisfied_headers.join(", "))).to_compile_error());
    }
    */

    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::ToCef for #name #ty_generics #where_clause {
            fn to_cef(&self) -> rust_cef::CefResult {
                let mut cef_entry = String::new();
                cef_entry.push_str("CEF:");
                cef_entry.push_str(self.cef_header_version());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_device_vendor());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_device_product());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_device_version());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_device_event_class_id());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_name());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_header_severity());
                cef_entry.push_str("|");
                cef_entry.push_str(self.cef_extensions());

                Ok(cef_entry)
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
    if !valid_header(&pathname) {
        return (SynError::new(nv.span(), CEF_INVALID_HEADER.as_str()).to_compile_error(), true)
    }

    let value = match nv.lit {
        Lit::Str(s) => s,
        _ => return (SynError::new(nv.span(), CEF_FIXED_HEADERS_VALUE_MUST_BE_STRING.as_str()).to_compile_error(), true),
    };

    let trait_name = format_ident!("CefHeader{}", pathname);
    let method_name = format_ident!("cef_header_{}", snakecase::to_snake_case(pathname.to_string().as_str()));

    (quote! {
        impl #impl_generics rust_cef::#trait_name for #name #ty_generics #where_clause {
            fn #method_name() -> rust_cef::CefResult {
                Ok(#value)
            }
        }
    }, false)
}
