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
use std::collections::HashMap;
use syn::{Attribute, MetaNameValue, Field, Fields, Data, DeriveInput, Error as SynError, Lit, Meta, NestedMeta, Ident, Variant, DataEnum, DataStruct};

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

    static ref CEF_HEADER_MISSING_VALUES_OR_INHERIT: String = "Deriving this trait requires a value for the header be provided through one of 'cef_values' or 'cef_inherit' macros on members of structs, and every variant of an enum.".to_owned();

    static ref CEF_INHERIT_STRUCT_USAGE: String = "'cef_inherit' macro should apply only to a struct field (not the struct itself), and specify the trait(s) to inherit from the field: #[cef_inherit(headerTrait)] ".to_owned();
    static ref CEF_VALUES_STRUCT_USAGE: String = "'cef_values' macro expects header values to be listed in the following syntax: #[cef_values(header1 = \"value1\", header2 = \"value2\", ...)] ".to_owned();
    static ref CEF_FIELD_STRUCT_USAGE: String = "'cef_field' macro adapts the attributed field using the fmt::Display trait into a CEF header trait. Use it on any field that implements fmt::Display: #[cef_field(headerTrait)]".to_owned();

    static ref CEF_INHERIT_ENUM_USAGE: String = "'cef_inherit' macro on an Enum::Variant should provide the enum field name (for struct variants) as a string literal or a 0-based index (for tuple variants) in the following format: #[cef_inherit(HeaderName1 = 0)] OR #[cef_inherit(HeaderName1 = \"field1\")]".to_owned();
    static ref CEF_VALUES_ENUM_USAGE: String = "'cef_values' macro on an Enum::Variant should provide a string literal in the following format: #[cef_inherit(HeaderName1 = \"value1\")]".to_owned();
    static ref CEF_FIELD_ENUM_USAGE: String = "'cef_field' macro adapts the indexed field using the fmt::Display trait into a CEF header trait. It may take a parameter indicating the integer index of the field to adapt, or the field name as a string literal: #[cef_field(HeaderName1 = 0)] OR #[cef_inherit(HeaderName1 = \"field1\")]".to_owned();

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

#[proc_macro_derive(CefHeaderVersion, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_version(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderVersion", &item)
}

#[proc_macro_derive(CefHeaderDeviceVendor, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_vendor(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceVendor", &item)
}

#[proc_macro_derive(CefHeaderDeviceProduct, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_product(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceProduct", &item)
}

#[proc_macro_derive(CefHeaderDeviceVersion, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_version(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceVersion", &item)
}
#[proc_macro_derive(CefHeaderDeviceEventClassID, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_device_event_class_id(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderDeviceEventClassID", &item)
}
#[proc_macro_derive(CefHeaderName, attributes(cef_values, cef_inherit, cef_field))]
pub fn derive_cef_header_name(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    implement_trait("CefHeaderName", &item)
}
#[proc_macro_derive(CefHeaderSeverity, attributes(cef_values, cef_inherit, cef_field))]
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
    if trait_name_str == "CefHeaderName" {
        println!("{:#?}", value.to_string());
    }

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
            header_value_from_child_struct(header_name, method_name, s, item)
        },
        Data::Enum(e) => {
            header_value_from_child_enum(header_name, method_name, e, item)
        },
        Data::Union(_) => return SynError::new(Span::call_site(), CEF_HEADERS_APPLICATION.to_owned()).to_compile_error(),
    }
}


fn header_value_from_child_struct(header_name: &Ident, method_name: &Ident, s: &DataStruct, item: &DeriveInput) -> TokenStream2 {
    let mut trait_values: Vec<TraitValue> = vec![];

    // look for fixed cef_values in top-level
    if let Some(ts) = top_level_cef_values(header_name, item, &mut trait_values) {
        return ts;
    }

    // now look for struct's field attributes
    for (index, field) in s.fields.iter().enumerate() {
        for attr in &field.attrs {
            if attr.path.is_ident("cef_inherit") || attr.path.is_ident("cef_field") {
                let usage_message = match attr.path.is_ident("cef_inherit") {
                    true => CEF_INHERIT_STRUCT_USAGE.to_owned(),
                    false => CEF_FIELD_STRUCT_USAGE.to_owned(),
                };

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
                                                rust_cef::#header_name::#method_name(self.#field_name)
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
                                    _ => return SynError::new(attr.span(), usage_message).to_compile_error(),
                                },
                                _ => return SynError::new(attr.span(), usage_message).to_compile_error(),
                            }
                        },
                        _ => return SynError::new(attr.span(), usage_message).to_compile_error(),
                    },
                    Err(e) => return e.to_compile_error(),
                }
            }
        }
    }

    match trait_values.len() {
        0 => return SynError::new(Span::call_site(), CEF_HEADER_MISSING_VALUES_OR_INHERIT.to_owned()).to_compile_error(),
        1 => return match trait_values.pop() {
            Some(tv) => tv.ts,
            None => return SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error(),
        },
        _ => {
            let errs = trait_values.iter().map(|tv| {
                return SynError::new(tv.span, format!("Trait {} had values provided in multiple places. Please remove all but one of these.", header_name))
                    .to_compile_error();
            });

            quote!{
                #(#errs)*
            }
        },
    }
}

fn header_value_from_child_enum(header_name: &Ident, method_name: &Ident, e: &DataEnum, item: &DeriveInput) -> TokenStream2 {
    let mut trait_values: Vec<TraitValue> = vec![];

    // look for fixed cef_values in top-level
    if let Some(ts) = top_level_cef_values(header_name, item, &mut trait_values) {
        return ts;
    }

    if let Some(ts) = all_variants_cef_value(header_name, method_name, &item.ident, &e, &mut trait_values) {
        return ts;
    }

    match trait_values.len() {
        0 => return SynError::new(Span::call_site(), CEF_HEADER_MISSING_VALUES_OR_INHERIT.to_owned()).to_compile_error(),
        1 => return match trait_values.pop() {
            Some(tv) => tv.ts,
            None => return SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error(),
        },
        _ => {
            let errs = trait_values.iter().map(|tv| {
                return SynError::new(tv.span, format!("Trait {} had values provided in multiple places. Please remove all but one of these.", header_name))
                    .to_compile_error();
            });

            quote!{
                #(#errs)*
            }
        },
    }
}

fn all_variants_cef_value(header_name: &Ident, method_name: &Ident, enum_name: &Ident, e: &DataEnum, trait_values: &mut Vec<TraitValue>) -> Option<TokenStream2> {

    let mut variant_values = HashMap::<Variant, TokenStream2>::new();

    // now look for struct's variants and get the header value for either ALL variants or NO variants
    for variant in e.variants.iter() {
        if let Some(ts) = variant_value(header_name, method_name, variant, &mut variant_values) {
            return Some(ts);
        }
    }

    if variant_values.len() > 0 && variant_values.len() < e.variants.len() {
        return Some(SynError::new(Span::call_site(), format!("For this enum, not all variants implement the trait {}. For Enum traits, each and every variant must supply a value, either through 'cef_values' or 'cef_inherit' macros.", header_name)).to_compile_error());
    }

    if variant_values.len() == e.variants.len() {
        let mut match_branches: Vec<TokenStream2> = vec![];

        for (variant, val) in variant_values {
            let ident = variant.ident;

            let variant_field_match = match variant.fields {
                Fields::Named(namedf) => {
                    let mut named_fields: Vec<TokenStream2> = vec![];
                    for f in namedf.named {
                        match f.ident {
                            None => return Some(SynError::new(f.span(), format!("Expected this to be a named field for generating Cef Headers.")).to_compile_error()),
                            Some(id) => {
                                let ignore_id = format_ident!("_{}", id);
                                named_fields.push(quote! {#id: #ignore_id});
                            },
                        }
                    }

                    quote! {
                        {#(#named_fields),*}
                    }
                },
                Fields::Unnamed(unnamedf) => {
                    let mut unnamed_fields: Vec<TokenStream2> = vec![];
                    for (idx, _) in unnamedf.unnamed.iter().enumerate() {
                        let ident = format_ident!("_ident{}", idx);
                        unnamed_fields.push(quote! {#ident});
                    }

                    quote! {
                        (#(#unnamed_fields),*)
                    }
                },
                Fields::Unit => quote!{
                },
            };

            let match_branch = quote! {
                #enum_name::#ident#variant_field_match => #val,
            };

            match_branches.push(match_branch);
        }

        let ts = quote! {
            match self {
                #(#match_branches)*
            }
        };

        let tv = TraitValue {
            ts,
            span: Span::call_site(),
        };

        trait_values.push(tv);
    }

    None
}

fn variant_value(header_name: &Ident, method_name: &Ident, variant: &Variant, variant_values: &mut  HashMap<Variant, TokenStream2>) -> Option<TokenStream2> {
    let mut in_variant_values: Vec<TraitValue> = vec![];

    for attr in &variant.attrs {
        if attr.path.is_ident("cef_inherit") || attr.path.is_ident("cef_values") || attr.path.is_ident("cef_field") {
            let usage_message = match attr.path.is_ident("cef_inherit") {
                true => CEF_INHERIT_ENUM_USAGE.to_owned(),
                false => match attr.path.is_ident("cef_values") {
                    true => CEF_VALUES_ENUM_USAGE.to_owned(),
                    false => CEF_FIELD_ENUM_USAGE.to_owned(),
                },
            };

            match attr.parse_meta() {
                Ok(parsed_meta) => match parsed_meta {
                    Meta::List(list) => for nested_meta in list.nested {
                        match nested_meta {
                            NestedMeta::Meta(meta) => match meta {
                                Meta::NameValue(nv) => if nv.path.is_ident(header_name) {
                                    // we found the attribute for this header
                                    if let Some(ts) = variant_trait(header_name, method_name, attr, nv, &variant.fields, &mut in_variant_values) {
                                        return Some(ts);
                                    }
                                },
                                _ => return Some(SynError::new(attr.span(), usage_message).to_compile_error()),
                            },
                            _ => return Some(SynError::new(attr.span(), usage_message).to_compile_error()),
                        }
                    },
                    _ => return Some(SynError::new(attr.span(), usage_message).to_compile_error()),
                },
                Err(e) => return Some(e.to_compile_error()),
            }
        }
    }

    if in_variant_values.len() > 1 {
        let errs = in_variant_values.iter().map(|tv| {
            return SynError::new(tv.span, format!("Trait {} had values provided in multiple places for variant {}. Please remove all but one of these.", header_name, variant.ident))
                .to_compile_error();
        });

        return Some(quote!{
            #(#errs)*
        });
    } else if in_variant_values.len() == 1 {
        match in_variant_values.pop() {
            Some(tv) => {variant_values.insert(variant.clone(), tv.ts);},
            None => return Some(SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error()),
        }
    }

    None
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

fn variant_trait(header_name: &Ident, method_name: &Ident, attr: &Attribute, nv: MetaNameValue, fields: &Fields, in_variant_values: &mut Vec<TraitValue>) -> Option<TokenStream2> {
    let ts = match &nv.lit {
        Lit::Int(litint) => match litint.base10_parse::<usize>() {
            Ok(i) => match fields.iter().nth(i) {
                //let's find the field index - only if necessary
                Some(field) => match &field.ident {
                    Some(ident) => return Some(SynError::new(nv.span(), format!("Field at index {} has a name '{}'. Don't use indexes for when not needed..", i, ident)).to_compile_error()),
                    None => {
                        let field_index = format_ident!("_ident{}", i);

                        match attr.path.is_ident("cef_inherit") {
                            // are we inheriting or field fmt::Display'ing?
                            true => quote! {
                                Ok(rust_cef::#header_name::#method_name(&#field_index))
                            },
                            false => match attr.path.is_ident("cef_field") {
                                true => quote! {
                                    Ok(format!("{}", #field_index).to_owned())
                                },
                                false => return Some(SynError::new(nv.lit.span(), CEF_VALUES_ENUM_USAGE.to_owned()).to_compile_error()),
                            },
                        }
                    },
                },
                None => return Some(SynError::new(nv.span(), format!("Field at index {} doesn't exist in this enum variant.", i)).to_compile_error()),
            },
            Err(e) => return Some(SynError::new(nv.lit.span(), format!("Value for index {} couldn't be parsed into an integer due to error {}", litint, e)).to_compile_error()),
        },
        Lit::Str(s) => match attr.path.is_ident("cef_values") {
            true => quote! { Ok(#s.to_owned())},
            false => {
                let mut maybe_field: Vec<&Field> = fields.iter().filter(|field| field.ident.is_some() && field.ident.as_ref().unwrap().to_string() == s.value()).collect();
                if maybe_field.len() > 1 {
                    return Some(SynError::new(nv.lit.span(), format!("More than one field with name {} was found.", s.value())).to_compile_error());
                }
                if maybe_field.len() == 0 {
                    return Some(SynError::new(nv.lit.span(), format!("No field with name {} was found.", s.value())).to_compile_error());
                }
                let fieldid = match maybe_field.pop() {
                    Some(field) => match &field.ident {
                        Some(ident) => format_ident!("_{}", ident),
                        None => return Some(SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error()),
                    },
                    None => return Some(SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error()),
                };

                match attr.path.is_ident("cef_field") {
                    true => quote! {
                            Ok(format!("{}", #fieldid).to_owned())
                        },
                    false =>  quote! {
                        rust_cef::#header_name::#method_name(&#fieldid)
                    },
                }
            },
        },
        _ => return Some(SynError::new(nv.lit.span(), format!("Value for {} can be a string literal (for direct placement), index, or field name.", header_name)).to_compile_error()),
    };

    let span = nv.span().clone();

    let tv = TraitValue {
        ts,
        span,
    };

    in_variant_values.push(tv);

    None
}