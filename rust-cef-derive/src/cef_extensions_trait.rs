/// Copyright 2020 Polyverse Corporation
///
/// This module provides functions to implement the CefExtensions trait
use crate::helpers::{is_valid_item_type, CEF_ATTRIBUTE_APPLICATION};
use crate::proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::convert::From;
use std::fmt::Display;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error as SynError, Field, Fields, Ident,
    Index, Lit, Meta, MetaNameValue, NestedMeta, Path, Variant,
};

const CEF_EXT_GOBBLE_APPLICABLE: &str = "'cef_ext_gobble' macro may only apply on fields (named or unnamed) but never on a struct or enum type, or enum variants.";
const CEF_EXT_FIELD_APPLICABLE: &str = "'cef_ext_field' macro may only apply on fields (named or unnamed) but never on a struct or enum type, or enum variants.";

const CEF_EXT_GOBBLE_USAGE: &str = "'cef_ext_gobble' macro must supply no arguments and appear by itself to inform CefExtensions derivation to gobble any keys generated by that field type's CefExtensions implementation. #[cef_ext_gobble]";
const CEF_EXT_FIELD_USAGE: &str = "'cef_ext_field' macro may optionally supply one argument which is the custom extension key name to use. If no arguments are supplied, the field's name is used. #[cef_ext_field(rename)]";

enum FieldValueType {
    GobbleTrait,
    DisplayTrait,
}

enum PrefixSelf {
    Yes,
    No,
}

#[derive(PartialEq)]
enum FieldNameFromId {
    Allowed,
    NotAllowed,
}

enum FieldIdentity {
    Ident(Ident),
    Index(syn::Index),
}

struct TraitValue {
    pub ts: TokenStream2,
    pub span: Span,
}

type CompileResult = Result<TokenStream2, TokenStream2>;
type CollectedCompileResult = Result<Vec<TokenStream2>, TokenStream2>;
type OptionalCompileResult = Result<Option<TokenStream2>, TokenStream2>;
type OptionalCollectedCompileResult = Result<Vec<Option<TokenStream2>>, TokenStream2>;

type VariantFieldResult = Result<Ident, TokenStream2>;

type ParseAttrResult<T> = Result<T, TokenStream2>;

/// Implements the trait asked by any of the `#[derive(CefHeader*)]` attributes
/// It creates the trait skeleton and outsources the returned value
/// to a child-item function.
pub fn implement_extensions_trait(item_tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item_tokens as DeriveInput);

    // Only applies to structs and enums
    if let Some(compile_error) = is_valid_item_type(&item) {
        return compile_error;
    }

    // type name
    let item_name = &item.ident;

    // generics
    let item_generics = &item.generics;
    let (item_impl_generics, item_ty_generics, item_where_clause) = item_generics.split_for_impl();

    let collections = extensions_from_child_item(&item);

    let trait_impl = quote! {
        impl #item_impl_generics rust_cef::CefExtensions for #item_name #item_ty_generics #item_where_clause {
            fn cef_extensions(&self, &mut collector: std::collections::HashMap<String, String>) -> CefExtensionsResult {
                #collections

                // let collections return errors if they wish
                Ok(())
            }
        }
    };

    println!("{:#?}", trait_impl.to_string());

    TokenStream::from(trait_impl)
}

/// This function provides the crucial value that
/// the implementing trait returns for the given item
///
/// Depending on whether the item is a Struct or an Enum,
/// the interpretation of other helper attributes changes.
/// This function does that detection and forks processing
/// to more specialized functions for each.
///
/// NOTE: Union types are not supported.
///
fn extensions_from_child_item(item: &DeriveInput) -> TokenStream2 {
    // Is the Item a struct or enum?
    match &item.data {
        Data::Struct(s) => extensions_from_child_struct(s, item),
        //Data::Enum(e) => extensions_from_child_enum(header_name, method_name, e, item),
        _ => {
            return SynError::new(Span::call_site(), CEF_ATTRIBUTE_APPLICATION.to_owned())
                .to_compile_error()
        }
    }
}

/// This function generates a CefExtensions trait on a struct,
/// picking (or gobbling) fields from within the struct,
/// or provided by CefExtensions traits downstream.
///
/// For a struct, extensions come in two primary ways:
///
/// `#[cef_ext_gobble]`
/// This gobbles any extensions added by that field's CefExtensions implementation.
///
/// This looks like
/// ```ignore
/// #[derive(CefExtensions)]
/// struct Outer {
///     #[cef_ext_gobble]
///     pub inner: Inner;
///
///     #[cef_ext_gobble]
///     pub inner2: Inner;
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl CefExtensions for Outer {
///     cef_extensions(&self) {
///         [
///             rust_cef::CefExtensions::cef_extensions(&self.inner)?,
///             rust_cef::CefExtensions::cef_extensions(&self.inner1)?,
///         ].join(" ")
///     }
/// }
/// ```
///
/// `#[cef_ext_field(optional_rename)]`
/// This adds an extension with the field name, or an optional custom name argument provided,
/// and uses the field's Display trait to provide the value.
///
/// This looks like
/// ```ignore
/// #[derive(CefExtensions)]
/// struct Outer {
///     #[cef_ext_field]
///     pub inner: Inner;
///
///     #[cef_ext_field(outer)]
///     pub inner2: Inner;
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl CefExtensions for Outer {
///     cef_extensions(&self) {
///         [
///             format!("inner = {}", &self.inner),
///             format!("outer = {}", &self.inner1),
///         ].join(" ")
///     }
/// }
/// ```
///
/// Of course they may be mixed:
/// ```ignore
/// #[derive(CefExtensions)]
/// struct Outer {
///     #[cef_ext_gobble]
///     pub inner: Inner;
///
///     #[cef_ext_field(outer)]
///     pub inner2: Inner;
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl CefExtensions for Outer {
///     cef_extensions(&self) {
///         [
///             rust_cef::CefExtensions::cef_extensions(&self.inner)?,
///             format!("outer = {}", &self.inner1),
///         ].join(" ")
///     }
/// }
/// ```
///
fn extensions_from_child_struct(s: &DataStruct, item: &DeriveInput) -> TokenStream2 {
    // Map all possible fields into expressions for adding to extensions
    let extension_exprs_result: OptionalCollectedCompileResult = s
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_identity = match &field.ident {
                Some(ident) => FieldIdentity::Ident(ident.clone()),
                None => FieldIdentity::Index(syn::Index::from(index)),
            };

            // look for field attributes
            field_extraction(
                &field.attrs,
                field_identity,
                FieldNameFromId::Allowed,
                &PrefixSelf::Yes,
                field.span(),
            )
        })
        .collect();

    let extension_exprs: Vec<TokenStream2> = match extension_exprs_result {
        Err(e) => return e,

        // optional ts has type Vec<Option<TokenStream2>>
        Ok(optionalts) => optionalts.into_iter().filter_map(|ots| ots).collect(),
    };

    let size: usize = extension_exprs.len();

    let extensions_impl = quote! {
        #(#extension_exprs)*
    };

    //println!("ExtensionsImpl ====> {:#?}", &extensions_impl.to_string());

    extensions_impl
}

// Helps cut through a lot of parse tree and doesn't confuse reading-context
fn parse_attrs_to_path(attr: &Attribute, messsage: &str) -> ParseAttrResult<Option<String>> {
    match attr.parse_meta() {
        Ok(parsed_meta) => match parsed_meta {
            Meta::Path(_) => Ok(None),
            Meta::List(ml) => match ml.nested.len() {
                0 | 1 => match ml.nested.first() {
                    None => Ok(None),
                    Some(nm) => match nm {
                        NestedMeta::Meta(m) => match m {
                            Meta::Path(p) => match p.get_ident() {
                                Some(ident) => Ok(Some(ident.to_string())),
                                _ => {
                                    return Err(
                                        SynError::new(attr.span(), messsage).to_compile_error()
                                    )
                                }
                            },
                            _ => {
                                return Err(SynError::new(attr.span(), messsage).to_compile_error())
                            }
                        },
                        _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
                    },
                },
                _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
            },
            _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
        },
        Err(e) => return Err(e.to_compile_error()),
    }
}

/// This function generates a CefExtensions trait on an Enum,
/// picking (or gobbling) fields from within the struct,
/// or provided by CefExtensions traits downstream.
///
/// For an Enum, extensions come in two primary ways:
///
///
/// `#[cef_ext_gobble]`
/// This gobbles any extensions added by that field's CefExtensions implementation.
///
/// AND
///
/// `#[cef_ext_field(optional_rename)]`
/// This adds an extension with the field name, or an optional custom name argument provided,
/// and uses the field's Display trait to provide the value.
///
/// This looks like
/// ```ignore
/// #[derive(CefExtensions)]
/// enum Items {
///     Variant1(
///         // unnamed fields need a name to be in CEF Extensions
///         #[cef_ext_field(age)]
///         int,
///
///         #[cef_ext_gobble]
///         HeaderImplementer1
///     ),
///
///     Variant2{
///         #[cef_ext_field]
///         name: String,
///
///         // this means a field called address is added with value address.to_string
///         #[cef_ext_field]
///         address: HeaderImplementer2
///     },
/// }
///
/// // where
/// impl CefExtensions for HeaderImplementer1 {
///     //...
/// }
/// // and
/// impl CefExtensions for HeaderImplementer2 {
///     //...
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl CefExtensions for Items {
///     method(&self) {
///         match &self {
///             Self::Variant1(_index0, _index1) => Ok([CefExtenions::cef_extensions(&_index1)?].join(" ")),
///             Self::Variant2{name: _name, address: _address} => Header::method(_address),
///         }
///     }
/// }
/// ```
///
///
/// Finally and for completeness, remember it is possible to mix-and-match
///
/// For Example:
/// ```ignore
/// #[derive(Header)]
/// enum Items {
///
///     Variant1(
///         int,
///
///         #[cef_field(Header)]
///         DisplayImplementer1
///     ),
///
///     Variant2{
///         name: String,
///
///         #[cef_inherit(Header)]
///         address: HeaderImplementer2
///     },
///
///     #[cef_value(Header = "Variant3")]
///     Variant3,
/// }
///
/// // where
/// impl Display for DisplayImplementer1 {
/// //...
/// }
/// // and
/// impl Header for HeaderImplementer2 {
/// //...
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl Header for Items {
///     method(&self) {
///         match &self {
///             Items::Variant1(_index0, _index1) => format!("{}",_index1),
///             Items::Variant2{name: _name, address: _address} => Header::method(_address),
///             Items::Variant3 => "variant3",
///         }
///     }
/// }
/// ```
///
/// NOTE: This method looks for ALL possible values first,
/// and then if only one is found, uses it. If no values are found,
/// an error is thrown, and if multiple values are found an error is
/// thrown to indicate conflict and ambiguity.
///
fn header_value_from_child_enum(e: &DataEnum, item: &DeriveInput) -> TokenStream2 {
    let match_branches_result: OptionalCollectedCompileResult = e
        .variants
        .iter()
        .map(|variant| destructure_and_match_variant(&variant))
        .collect();

    let match_branches: Vec<TokenStream2> = match match_branches_result {
        Ok(tses) => tses.into_iter().filter_map(|v| v).collect(),
        Err(ts) => return ts,
    };

    // Finally compile all branches into a match
    // operator block like thus:
    //
    // match &self {
    //       variant1 branch => {add extensions for variant1},
    //       variant2 branch => {add extensions for variant2},
    // ...
    // }
    let ts = quote! {
        match &self {
            #(#match_branches)*
        }
    };

    ts
}

/// create a enum variant field de-structuring expression
/// and match the field which has an attribute for obtaining header
/// value.
///
///```ignore
///   // For named fields
///  {name: _name, address: _address} => {
///     // gobble cef extensions from address field
///     rust_cef_::CefExtensions::cef_extensions(&_address, &mut collector);
///     // add cef extensions from name field
///     collector.insert("newname", format!("{}", _name));
/// }
///  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///
///  (_index0, _index1) => {
///     // gobble cef extensions from _index0 unnamed field
///     rust_cef_::CefExtensions::cef_extensions(&_index0, &mut collector);
///     // add cef extensions from index1 unnamed field
///     collector.insert("newname", format!("{}", _index1));
/// }
///  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///```
///
/// The value part may be a Dispay trait formatted value (i.e. format!("{}", field))
/// or an inheritance of the header trait (i.e. header_trait::method(&field))
///
///
///
fn destructure_and_match_variant(variant: &Variant) -> OptionalCompileResult {
    // Get the identity of the Variant
    // This part:
    // ```
    // V1{...}
    // ^^
    // V2{...}
    // ^^
    // ```
    //
    let ident = variant.ident.clone();

    // create a field-capture
    // field_captures is a Vector of either:
    // `<None, None, None, Some(captureField: _captureField, // add _captureField to collector), None,>`
    // (for unnamed fields an index name is used to capture them and use them)
    //
    // if any field is named (and not ignored with an underscore), then the trait_values vector
    // will have a tokenstream for that value
    //
    let field_extractions_result: Result<Vec<(TokenStream2, TokenStream2)>, TokenStream2> = variant
        .fields
        .iter()
        .enumerate()
        .map(
            |(index, f)| -> Result<(TokenStream2, TokenStream2), TokenStream2> {
                // see if there's any field-level cef_inherit or cef_field attributes on the variant

                let (field_prefix, fieldid) = match &f.ident {
                    Some(id) => (quote! {#id:}, format_ident!("_{}", id)),
                    None => (quote! {}, format_ident!("_index{}", index)),
                };

                let (final_fieldid, extraction) = match field_extraction(
                    &f.attrs,
                    FieldIdentity::Ident(fieldid.clone()),
                    FieldNameFromId::NotAllowed,
                    &PrefixSelf::No,
                    f.span(),
                ) {
                    Err(ts) => return Err(ts),
                    Ok(maybe_ext) => match maybe_ext {
                        Some(ext) => (fieldid, ext),

                        // No extraction for this field
                        // first, capture fieldid as "_" to ignore it (good practice)
                        // and give it an empty extraction
                        None => (format_ident!("_"), quote! {}),
                    },
                };

                Ok((quote! {#field_prefix#final_fieldid}, extraction))
            },
        )
        .collect();

    let (field_captures, field_extractions): (Vec<_>, Vec<_>) = match field_extractions_result {
        Err(ts) => return Err(ts),
        Ok(fc) => fc.iter().cloned().unzip(),
    };

    // Named fields (aka Struct variant) is wrapped with {},
    // whereas Unnamed fields (aka Tuple variant) is wrapped with ()
    // Now we have something like:
    // `{_, _, _, captureField: _captureField, _}` - for named fields
    // `(_, _, _, _index3, _)` - for unnamed fields
    let variant_capture = match &variant.fields {
        Fields::Named(_) => quote! {{#(#field_captures),*}},
        Fields::Unnamed(_) => quote! {(#(#field_captures),*)},
        Fields::Unit => quote! {},
    };

    //
    // The overall match branch now looks like this:
    //
    // For named fields:
    // Self::Variant1{name: _name, address: _address}  => {
    //      collector.insert("name", format!("{}", _name));
    //      rust_cef::CefExtensions::cef_extensions(&_address, &mut collector);
    // },
    //
    // For unnamed fields:
    // Self::Variant1(_index0, _index1)  => {
    //      collector.insert("newname", format!("{}", _index0));
    //      rust_cef::CefExtensions::cef_extensions(&_index1, &mut collector);
    // },
    let match_branch = quote! {
        Self::#ident#variant_capture => {
            #(#field_extractions);*
        },
    };

    Ok(Some(match_branch))
}

fn field_extraction(
    attrs: &Vec<Attribute>,
    field_identity: FieldIdentity,
    field_name_from_id: FieldNameFromId,
    prefix_self: &PrefixSelf,
    span: Span,
) -> Result<Option<TokenStream2>, TokenStream2> {
    // look for field attributes
    let values_for_field_result: CollectedCompileResult = attrs.iter()
        .filter(|attr| attr.path.is_ident("cef_ext_gobble") || attr.path.is_ident("cef_ext_field"))
        .map(|attr| {
            let (usage_message, value_type) = match attr.path.is_ident("cef_ext_gobble") {
                true => (CEF_EXT_GOBBLE_USAGE.to_owned(), FieldValueType::GobbleTrait),
                false => (CEF_EXT_FIELD_USAGE.to_owned(), FieldValueType::DisplayTrait),
            };

            match parse_attrs_to_path(&attr, usage_message.as_str()) {
                Ok(None) => match &field_identity {
                    FieldIdentity::Ident(fieldid) => match value_type {
                        FieldValueType::GobbleTrait => Ok(field_value(fieldid.to_string().as_str(), fieldid, &value_type, prefix_self)),
                        FieldValueType::DisplayTrait if FieldNameFromId::Allowed == field_name_from_id => Ok(field_value(fieldid.to_string().as_str(), fieldid, &value_type, prefix_self)),
                        FieldValueType::DisplayTrait => Err(SynError::new(attr.span(), format!("'cef_ext_field' should have a single parameter with the field name when used on unnamed fields. Cannot use typle index as a cef key.")).to_compile_error()),
                    },
                    FieldIdentity::Index(index) => match value_type {
                        FieldValueType::GobbleTrait => Ok(field_value("ignored", index, &value_type, prefix_self)),
                        _ => Err(SynError::new(attr.span(), format!("'cef_ext_field' should have a single parameter with the field name when used on unnamed fields. Cannot use typle index as a cef key.")).to_compile_error()),
                    },
                },
                Ok(Some(newfield)) => match &field_identity {
                    FieldIdentity::Ident(fieldid) => Ok(field_value(newfield.as_str(), fieldid, &value_type, &PrefixSelf::Yes)),
                    FieldIdentity::Index(index) => match value_type {
                        FieldValueType::GobbleTrait => Ok(field_value("ignored", index, &value_type, prefix_self)),
                        FieldValueType::DisplayTrait => Ok(field_value(newfield.as_str(), index, &value_type, prefix_self)),
                    },
                },
                Err(e) => return Err(e),
            }
        }).collect();

    match values_for_field_result {
        Ok(mut values_for_field) => match values_for_field.len() {
            0 | 1 => Ok(values_for_field.pop()),
            _ => Err(SynError::new(
                span,
                format!("Multiple values for CefExtensions found for field").to_owned(),
            )
            .to_compile_error()),
        },
        Err(e) => Err(e),
    }
}

/// Generates a value from a field
fn field_value<T: quote::ToTokens>(
    field_name: &str,
    field_ident: T,
    value_type: &FieldValueType,
    prefix_self: &PrefixSelf,
) -> TokenStream2 {
    let maybe_self = match prefix_self {
        PrefixSelf::Yes => quote! {&self.},
        PrefixSelf::No => quote! {},
    };

    match value_type {
        FieldValueType::GobbleTrait => quote! {
            if let Err(err) = rust_cef::CefExtensions::cef_extensions(#maybe_self#field_ident, &mut collector) {
                return Err(err);
            }
        },
        FieldValueType::DisplayTrait => quote! {
            collector.insert(#field_name.to_owned(), format!("{}", #maybe_self#field_ident));
        },
    }
}
