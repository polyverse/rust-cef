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

use crate::proc_macro::TokenStream;
use inflections::case::to_snake_case;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::convert::From;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error as SynError, Lit, Meta, NestedMeta, Ident};

macro_rules! parse_nonempty_attribute_args {
    ($attr:ident, $errormsg:expr) => {
            match $attr.is_empty() {
                true => return TokenStream::from(SynError::new(Span::call_site(), $errormsg).to_compile_error()),
                false => match parse_macro_input!($attr as AttributeArgs) {
                    a if a.len() > 0 => a,
                    _ => return TokenStream::from(SynError::new(Span::call_site(), $errormsg).to_compile_error()),
                }
            }
    }
}

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
    static ref CEF_HEADERS_APPLICATION: String = "This attribute only applies to Structs or Enums.".to_owned();

    static ref CEF_HEADER_MISSING_VALUES_OR_INHERIT: String = "Deriving this trait requires a value for the header be provided through one of 'cef_values' or 'cef_inherit' macros.".to_owned();

    static ref CEF_INHERIT_STRUCT_USAGE: String = "'cef_inherit' macro should apply only to a struct field (not the struct itself), and specify the trait(s) to inherit from the field: #[cef_inherit(headerTrait)] ".to_owned();

    static ref CEF_VALUES_STRUCT_USAGE: String = "'cef_values' macro expects header values to be listed in the following syntax: #[cef_values(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_VALUES_STRINGS: String = "'cef_values' macro expects all values to be string literals".to_owned();
}

struct TraitValue {
    pub ts: TokenStream2,
    pub span: Span,
}

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

#[proc_macro_derive(CefHeaderVersion, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_version(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderVersion", &item)
}

#[proc_macro_derive(CefHeaderDeviceVendor, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_device_vendor(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceVendor", &item)
}

#[proc_macro_derive(CefHeaderDeviceProduct, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_device_product(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceProduct", &item)
}

#[proc_macro_derive(CefHeaderDeviceVersion, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_device_version(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceVersion", &item)
}
#[proc_macro_derive(CefHeaderDeviceEventClassID, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_device_event_class_id(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceEventClassID", &item)
}
#[proc_macro_derive(CefHeaderName, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_name(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderName", &item)
}
#[proc_macro_derive(CefHeaderSeverity, attributes(cef_values, cef_inherit))]
pub fn derive_cef_header_severity(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderSeverity", &item)
}


#[proc_macro_derive(CefExtensions)]
pub fn derive_cef_extensions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // default implementation is great
    let cef_extensions_impl = quote! {
        impl #impl_generics rust_cef::CefExtensions for #name #ty_generics #where_clause {
            fn cef_extensions(&self) -> rust_cef::CefResult {
                Ok("".to_owned())
            }
        }
    };

    TokenStream::from(cef_extensions_impl)
}

fn is_valid_header(id: &str) -> bool {
    for header in CEF_ALLOWED_HEADERS.iter() {
        if id == *header {
            return true;
        }
    }

    false
}

fn is_valid_item_type(item: &DeriveInput) -> Option<TokenStream> {
        // Only applies to structs and enums
        match item.data {
            Data::Struct(_) | Data::Enum(_) => {}
            _ => {
                return Some(TokenStream::from(
                    SynError::new(Span::call_site(), CEF_HEADERS_APPLICATION.to_owned())
                        .to_compile_error(),
                ))
            }
        }

        None
}

fn implement_trait(trait_name_str: &str, item: &DeriveInput) -> TokenStream {
    // type name
    let item_name = &item.ident;

    // generics
    let item_generics = &item.generics;
    let (item_impl_generics, item_ty_generics, item_where_clause) = item_generics.split_for_impl();

    let trait_name = format_ident!("{}", trait_name_str);
    let method_name = format_ident!("{}",
        to_snake_case(trait_name.to_string().as_str())
    );

    let value = header_value_from_child_item(&trait_name, &method_name, &item);

    TokenStream::from(quote! {
        impl #item_impl_generics rust_cef::#trait_name for #item_name #item_ty_generics #item_where_clause {
            fn #method_name(&self) -> rust_cef::CefResult {
                #value
            }
        }
    })
}

fn header_value_from_child_item(header_name: &Ident, method_name: &Ident, item: &DeriveInput) -> TokenStream2 {
    // Is the Item a struct or enum?
    match &item.data {
        Data::Struct(s) => {
            header_value_from_child_struct(header_name, method_name, item)
        },
        Data::Enum(e) => {
            header_value_from_child_enum(header_name, method_name, item)
        },
        Data::Union(_) => return SynError::new(Span::call_site(), CEF_HEADERS_APPLICATION.to_owned()).to_compile_error(),
    }
}


fn header_value_from_child_struct(header_name: &Ident, method_name: &Ident, item: &DeriveInput) -> TokenStream2 {
    let s = match &item.data {
        Data::Struct(s) => s,
        _ => panic!("header_value_from_child_struct should never be called for a non-struct item."),
    };

    let mut trait_values: Vec<TraitValue> = vec![];

    // look for fixed cef_values in top-level
    if let Some(ts) = top_level_cef_values(header_name, item, &mut trait_values) {
        return ts;
    }

    // now look for struct's field attributes
    for (index, field) in s.fields.iter().enumerate() {
        for attr in &field.attrs {
            if attr.path.is_ident("cef_inherit") || attr.path.is_ident("cef_values") {
                match attr.parse_meta() {
                    Ok(parsed_meta) => match parsed_meta {
                        Meta::List(list) => for nested_meta in list.nested {
                            match nested_meta {
                                NestedMeta::Meta(meta) => match meta {
                                    Meta::Path(p) => if p.is_ident(header_name) {

                                        let field_name = match &field.ident {
                                            Some(i) => format_ident!("{}", i),
                                            None => format_ident!("{}", index),
                                        };

                                        // let's use our field's value.. either through format! or the trait
                                        let ts = match attr.path.is_ident("cef_inherit") {
                                            true => quote!{
                                                rust_cef::#header_name::#method_name(&self)
                                            },
                                            false => quote!{
                                                Ok(format!("{}", self.#field_name))
                                            },
                                        };

                                        let span = p.span().clone();

                                        let tv = TraitValue {
                                            ts,
                                            span,
                                        };

                                        trait_values.push(tv);
                                    },
                                    _ => return SynError::new(attr.span(), CEF_INHERIT_STRUCT_USAGE.to_owned()).to_compile_error(),
                                },
                                _ => return SynError::new(attr.span(), CEF_INHERIT_STRUCT_USAGE.to_owned()).to_compile_error(),
                            }
                        },
                        _ => return SynError::new(attr.span(), CEF_INHERIT_STRUCT_USAGE.to_owned()).to_compile_error(),
                    },
                    Err(e) => return SynError::new(attr.span(), CEF_INHERIT_STRUCT_USAGE.to_owned()).to_compile_error(),
                }
            }
        }
    }

    if trait_values.len() == 0 {
        return SynError::new(Span::call_site(), CEF_HEADER_MISSING_VALUES_OR_INHERIT.to_owned()).to_compile_error();
    }

    if trait_values.len() == 1 {
        return match trait_values.pop() {
            Some(tv) => tv.ts,
            None => return SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error(),
        };
    }

    quote!{
        Ok("0".to_owned())
    }
}

fn header_value_from_child_enum(header_name: &Ident, method_name: &Ident, item: &DeriveInput) -> TokenStream2 {
    let e = match &item.data {
        Data::Enum(e) => e,
        _ => panic!("header_value_from_child_struct should never be called for a non-struct item."),
    };

    quote!{
        Ok("0".to_owned())
    }
}


fn top_level_cef_values(header_name: &Ident, item: &DeriveInput, trait_values: &mut Vec<TraitValue>) -> Option<TokenStream2> {
    for attr in &item.attrs {
        if attr.path.is_ident("cef_inherit") {
            return Some(SynError::new(attr.path.span(), CEF_INHERIT_STRUCT_USAGE.to_owned()).to_compile_error());
        }

        if attr.path.is_ident("cef_values") {
            match attr.parse_meta() {
                Err(e) => return Some(e.to_compile_error()),
                Ok(metadata) => match metadata {
                    Meta::List(list) => for nestedmeta in list.nested {
                        match nestedmeta {
                            NestedMeta::Meta(meta) => match meta {
                                Meta::NameValue(mnv) => {
                                    if mnv.path.is_ident(header_name) {
                                        match &mnv.lit {
                                            Lit::Str(strval) => {
                                                let ts = quote! {
                                                    Ok(#strval.to_owned())
                                                };
                                                let span = mnv.span().clone();
                                                trait_values.push(TraitValue{
                                                    ts,
                                                    span,
                                                });
                                            },
                                            _ => return Some(SynError::new(mnv.lit.span(), CEF_VALUES_STRINGS.to_owned()).to_compile_error()),
                                        }
                                    }
                                },
                                _ => return Some(SynError::new(attr.span(), CEF_VALUES_STRUCT_USAGE.to_owned()).to_compile_error()),
                            },
                            _ => return Some(SynError::new(attr.span(), CEF_VALUES_STRUCT_USAGE.to_owned()).to_compile_error()),
                        }
                    },
                    _ => return Some(SynError::new(attr.span(), CEF_VALUES_STRUCT_USAGE.to_owned()).to_compile_error()),
                }
            }
        }
    }

    None
}