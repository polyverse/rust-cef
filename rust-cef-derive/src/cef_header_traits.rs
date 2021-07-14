/// Copyright 2020 Polyverse Corporation
///
/// This module provides functions to implement the CefHeader* traits
use crate::helpers::{
    is_valid_item_type, parse_attrs_to_name_value, ParseAttrResult, CEF_ATTRIBUTE_APPLICATION,
};
use crate::proc_macro::TokenStream;
use inflections::case::to_snake_case;
use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::convert::From;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Error as SynError,
    Field, Fields, Ident, Index, Lit, Meta, NestedMeta, Path, Variant,
};

const CEF_ALLOWED_HEADERS: &[&str] = &[
    "Version",
    "DeviceVendor",
    "DeviceProduct",
    "DeviceVersion",
    "DeviceEventClassID",
    "Name",
    "Severity",
];

const CEF_HEADER_MISSING_VALUES_OR_INHERIT: &str = "Deriving this trait requires a value for the header be provided through one of 'cef_values' or 'cef_inherit' macros on members of structs, and every variant of an enum.";

const CEF_VALUES_APPLICABLE: &str =
    "'cef_values' macro may apply on a Struct, Enum or Enum::Variant, but never on fields";
const CEF_INHERIT_APPLICABLE: &str = "'cef_inherit' macro should apply only to a struct/tuple field (possibly inside an Enum variant), but not the Struct, Enum, or Enum::Variant.";
const CEF_FIELD_APPLICABLE: &str = "'cef_field' macro should apply only to a struct/tuple field (possibly inside an Enum variant), but not the Struct, Enum, or Enum::Variant.";

const CEF_VALUES_USAGE: &str = "'cef_values' macro expects header values to be listed in the following syntax: #[cef_values(header1 = \"value1\", header2 = \"value2\", ...)] ";
const CEF_INHERIT_USAGE: &str = "'cef_inherit' macro adapts the attributed field by inheriting the desired trait from that field: #[cef_inherit(headerTrait)] ";
const CEF_FIELD_USAGE: &str = "'cef_field' macro adapts the attributed field using the fmt::Display trait into a CEF header trait. Use it on any field that implements fmt::Display: #[cef_field(headerTrait)]";

const CEF_VALUES_STRINGS: &str = "'cef_values' macro expects all values to be string literals";

lazy_static! {
    static ref CEF_INVALID_HEADER: String = [
        "header name should be one of the following: ",
        CEF_ALLOWED_HEADERS.join(",").as_str()
    ]
    .join(" ");
}

enum FieldValueType {
    InheritTrait,
    DisplayTrait,
}

enum PrefixSelf {
    Yes,
    No,
}

struct TraitValue {
    pub ts: TokenStream2,
    pub span: Span,
}

type CompileResult = Result<TokenStream2, TokenStream2>;
type CollectedCompileResult = Result<Vec<Option<TokenStream2>>, TokenStream2>;
type OptionalCompileResult = Result<Option<TokenStream2>, TokenStream2>;

/// Implements the trait asked by any of the `#[derive(CefHeader*)]` attributes
/// It creates the trait skeleton and outsources the returned value
/// to a child-item function.
pub fn implement_header_trait(trait_name_str: &str, item_tokens: TokenStream) -> TokenStream {
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

    let trait_name = format_ident!("{}", trait_name_str);
    let method_name = format_ident!("{}", to_snake_case(trait_name.to_string().as_str()));

    let value = header_value_from_child_item(&trait_name, &method_name, &item);

    let trait_impl = quote! {
        impl #item_impl_generics rust_cef::#trait_name for #item_name #item_ty_generics #item_where_clause {
            fn #method_name(&self) -> rust_cef::CefResult {
                #value
            }
        }
    };

    //println!("{:#?}", trait_impl.to_string());

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
fn header_value_from_child_item(
    header_name: &Ident,
    method_name: &Ident,
    item: &DeriveInput,
) -> TokenStream2 {
    // Is the Item a struct or enum?
    match &item.data {
        Data::Struct(s) => header_value_from_child_struct(header_name, method_name, s, item),
        Data::Enum(e) => header_value_from_child_enum(header_name, method_name, e, item),
        _ => SynError::new(Span::call_site(), CEF_ATTRIBUTE_APPLICATION.to_owned())
            .to_compile_error(),
    }
}

/// This function retrievs the value for the header trait
/// from a Struct item
///
/// For a struct, v alues come from three primary ways:
///
/// `#[cef_values(HeaderName = value)]`
/// This sets the value as a constant string literal. This
/// attribute must be on the root the Struct only (not on fields or variants)
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// #[cef_values(Header = "value")]
/// struct Outer {
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl Header for Outer {
///     method(&self) {
///         "value"
///     }
/// }
/// ```
///
/// `#[cef_inherit(HeaderName)`
/// This sets the value of that header as an inherited trait from a field.
/// This property only applies to fields of a struct.
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// struct Outer {
///     #[cef_inherit(Header)]
///     pub inner: Inner;
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl Header for Outer {
///     method(&self) {
///         Trait::method(&self.inner)
///     }
/// }
/// ```
///
/// #[cef_field(HeaderName)]
/// This sets the value of that header as the default Display formatting for the field.
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// struct Outer {
///     #[cef_field(Header)]
///     pub inner: Inner;
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl Header for Outer {
///     method(&self) {
///         format!("{}", &self.inner)
///     }
/// }
/// ```
///
/// NOTE: This method looks for ALL possible values first,
/// and then if only one is found, uses it. If no values are found,
/// an error is thrown, and if multiple values are found an error is
/// thrown to indicate conflict and ambiguity.
///
fn header_value_from_child_struct(
    header_name: &Ident,
    method_name: &Ident,
    s: &DataStruct,
    item: &DeriveInput,
) -> TokenStream2 {
    let mut trait_values: Vec<TraitValue> = vec![];

    // look for fixed cef_values in top-level
    if let Some(ts) = top_level_cef_values(header_name, &item.attrs, &mut trait_values) {
        return ts;
    }

    // now look for struct's field attributes
    for (index, field) in s.fields.iter().enumerate() {
        for attr in &field.attrs {
            if attr.path.is_ident("cef_inherit") || attr.path.is_ident("cef_field") {
                let (usage_message, value_type) = match attr.path.is_ident("cef_inherit") {
                    true => (CEF_INHERIT_USAGE.to_owned(), FieldValueType::InheritTrait),
                    false => (CEF_FIELD_USAGE.to_owned(), FieldValueType::DisplayTrait),
                };

                match parse_attrs_to_path(attr, usage_message.as_str()) {
                    Ok(paths) => {
                        for p in paths {
                            if p.is_ident(header_name) {
                                let ts = match &field.ident {
                                    Some(i) => field_value(
                                        header_name,
                                        method_name,
                                        &value_type,
                                        format_ident!("{}", i),
                                        PrefixSelf::Yes,
                                    ),
                                    None => field_value(
                                        header_name,
                                        method_name,
                                        &value_type,
                                        Index::from(index),
                                        PrefixSelf::Yes,
                                    ),
                                };

                                let tv = TraitValue { ts, span: p.span() };

                                trait_values.push(tv);
                            }
                        }
                    }
                    Err(e) => return e,
                }
            }
        }
    }

    match trait_values.len() {
        0 => SynError::new(Span::call_site(), CEF_HEADER_MISSING_VALUES_OR_INHERIT.to_owned()).to_compile_error(),
        1 => match trait_values.pop() {
            Some(tv) => tv.ts,
            None => SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error(),
        },
        _ => {
            let errs = trait_values.iter().map(|tv|
                SynError::new(tv.span, format!("Trait {} had values provided in multiple places. Please remove all but one of these.", header_name))
                    .to_compile_error()
            );

            quote!{
                #(#errs)*
            }
        },
    }
}

/// This function retrievs the value for the header trait
/// from am Enum item, and is significantly more complicated
/// than the Struct implementation for a number of reasons.
///
/// First let's understand the overall concept of how Enums work:
/// 1. If an Enum exposes a Header, then each variant must provide a value
///    for that header. It must be exhaustive and comprehensive.
/// 2. If an Enum sets a Header at the root (using `#[cef_values(Header = "value")]`)
///    None of its variants must provide that value or override it.
///
/// Enum values come from three primary ways:
///
/// * `#[cef_values(HeaderName = value)]`
/// This sets the value as a constant string literal. This
/// attribute may be on the root the Enum (in which case any Variants may NOT override or conflit with it.)
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// #[cef_values(Header = "value")]
/// enum Items {
/// }
/// ```
///
/// and expands to:
/// ```ignore
/// impl Header for Items {
///     method(&self) {
///         "value"
///     }
/// }
/// ```
///
/// It may also be applied to variants, thereby allowing each variant to choose a different
/// value, or mix and match with `cef_inherit` and `cef_field`.
///
/// * `#[cef_inherit(HeaderName)]`
/// This attribute applies only to a field.
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// enum Items {
///     #[cef_inherit(Header = 1)]
///     Variant1(
///         int,
///
///         #[cef_inherit(Header)]
///         HeaderImplementer1
///     ),
///
///     Variant2{
///         name: String,
///
///         #[cef_inherit(Header)]
///         address: HeaderImplementer2
///     },
/// }
///
/// // where
/// impl Header for HeaderImplementer1 {
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
///             Items::Variant1(_index0, _index1) => Header::method(_index1),
///             Items::Variant2{name: _name, address: _address} => Header::method(_address),
///         }
///     }
/// }
/// ```
///
/// As you can see, the variants don't have to be symmetric, so long as they each provide
/// an implementation for the Header. It is also not necessary that all variants inhert it. Some may
/// inherit, while others may use field values.
///
/// However, they may not override the root-level `#[cef_values(Header = "value")]` construct which is a conflict.
///
/// #[cef_field(HeaderName = ["field" | index])]
/// This operates identically to #[cef_inherit], except that instead of calling the field's
/// trait method (thus inheriting the trait), it uses the Display trait to produce a string
///
/// Essentially, this looks like
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
///         #[cef_field(Header)]
///         address: DisplayImplementer2
///     },
/// }
///
/// // where
/// impl Display for DisplayImplementer1 {
/// //...
/// }
/// // and
/// impl Display for DisplayImplementer2 {
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
///             Items::Variant2{name: _name, address: _address} => format!("{}",_address),
///         }
///     }
/// }
/// ```
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
fn header_value_from_child_enum(
    header_name: &Ident,
    method_name: &Ident,
    e: &DataEnum,
    item: &DeriveInput,
) -> TokenStream2 {
    let mut trait_values: Vec<TraitValue> = vec![];

    // look for fixed cef_values in top-level
    if let Some(ts) = top_level_cef_values(header_name, &item.attrs, &mut trait_values) {
        return ts;
    }

    // Set CEF value for this header from every variant
    if let Some(ts) = all_variants_cef_value(header_name, method_name, &e, &mut trait_values) {
        return ts;
    }

    match trait_values.len() {
        0 => SynError::new(Span::call_site(), CEF_HEADER_MISSING_VALUES_OR_INHERIT.to_owned()).to_compile_error(),
        1 => match trait_values.pop() {
            Some(tv) => tv.ts,
            None => SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error(),
        },
        _ => {
            let errs = trait_values.iter().map(|tv|
                SynError::new(tv.span, format!("Trait {} had values provided in multiple places. Please remove all but one of these.", header_name))
                    .to_compile_error()
            );

            quote!{
                #(#errs)*
            }
        },
    }
}

/// This function creates a match statement with args for every variant for the Enum
/// this is what allows a unified Header trait to be implemented on the Enum.
///
fn all_variants_cef_value(
    header_name: &Ident,
    method_name: &Ident,
    e: &DataEnum,
    trait_values: &mut Vec<TraitValue>,
) -> Option<TokenStream2> {
    let match_branches_result: CollectedCompileResult = e
        .variants
        .iter()
        .map(|variant| destructure_and_match_variant(header_name, method_name, &variant))
        .collect();

    let match_branches: Vec<TokenStream2> = match match_branches_result {
        Ok(tses) => tses.into_iter().flatten().collect(),
        Err(ts) => return Some(ts),
    };

    // No implementations from variant
    if match_branches.is_empty() {
        return None;
    }

    // did we get ALL variants?
    if match_branches.len() < e.variants.len() {
        return Some(SynError::new(Span::call_site(), format!("Header trait {} was not implemented for ALL variants of this enum. Unable to derive for the overall enum.", header_name)).to_compile_error());
    }

    // Finally compile all branches into a match
    // operator block like thus:
    //
    // match &self {
    //       variant1 branch => value1,
    //       variant2 branch => value12,
    // ...
    // }
    let ts = quote! {
        match &self {
            #(#match_branches)*
        }
    };

    let tv = TraitValue {
        ts,
        span: Span::call_site(),
    };

    trait_values.push(tv);

    None
}

/// create a enum variant field de-structuring expression
/// and match the field which has an attribute for obtaining header
/// value.
///
/// create this field-matcher for named fields
///```ignore
///  {name: _name, address: _address} => _address
///  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///
///  (_index0, index1) => _index0
///  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///```
///
/// The value part may be a Dispay trait formatted value (i.e. format!("{}", field))
/// or an inheritance of the header trait (i.e. header_trait::method(&field))
///
///
///
fn destructure_and_match_variant(
    header_name: &Ident,
    method_name: &Ident,
    variant: &Variant,
) -> OptionalCompileResult {
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

    let mut trait_values: Vec<TraitValue> = vec![];

    // See if there's any top-level cef_values attributes on the variant
    if let Some(ts) = top_level_cef_values(header_name, &variant.attrs, &mut trait_values) {
        return Err(ts);
    }

    // create a field-capture
    // field_captures is a Vector of either:
    // `<_, _, _, captureField: _captureField, _,>` - for named fields
    // `<_, _, _, _index3, _,>` - for unnamed fields
    //
    // if any field is named (and not ignored with an underscore), then the trait_values vector
    // will have a tokenstream for that value
    //
    let field_captures_result: Result<Vec<TokenStream2>, TokenStream2> = variant
        .fields
        .iter()
        .enumerate()
        .map(|(index, f)| -> CompileResult {
            // see if there's any field-level cef_inherit or cef_field attributes on the variant
            let fieldid = match &f.ident {
                Some(id) => format_ident!("{}", id),
                None => format_ident!("index{}", index),
            };

            let final_fieldid =
                match variant_field_value(header_name, method_name, &fieldid, f, &mut trait_values)
                {
                    Err(ts) => return Err(ts),
                    Ok(ident) => ident,
                };

            Ok(quote! {#final_fieldid})
        })
        .collect();

    let field_captures = match field_captures_result {
        Err(ts) => return Err(ts),
        Ok(fc) => fc,
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

    //println!("\n\n{:#?}\n\n", variant_capture.to_string());

    let val = match trait_values.len() {
        // no values for this variant at this level. We return no branch.
        0 => return Ok(None),
        1 => match trait_values.pop() {
            Some(tv) => tv.ts,
            None => return Err(SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error()),
        },
        _ => {
            let errs = trait_values.iter().map(|tv|
                SynError::new(tv.span, format!("Trait {} had values provided in multiple places for variant {}. Please remove all but one of these.", header_name, ident))
                    .to_compile_error()
            );

            return Err(quote!{
                #(#errs)*
            });
        },
    };

    //
    // The overall match branch now looks like this:
    //
    // For named fields:
    // Self::Variant1{name: _name, address: _}  => format!("{}", _name)
    // ^^^^^  ^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^      ^^^^^^^^^^^^^^^^^^^^
    // enum   variant   field matchers we just created     The value for this variant (which may refer to the fields - even unnamed fields are captured under positional names _ident0, _ident1, etc.)
    //
    // For unnamed fields:
    // Self::Variant1(_index, _)  => format!("{}", _index0)
    // ^^^^^  ^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^      ^^^^^^^^^^^^^^^^^^^^
    // enum   variant   field matchers we just created     The value for this variant (which may refer to the fields - even unnamed fields are captured under positional names _ident0, _ident1, etc.)
    let match_branch = quote! {
        Self::#ident#variant_capture => #val,
    };

    Ok(Some(match_branch))
}

fn variant_field_value(
    header_name: &Ident,
    method_name: &Ident,
    fieldid: &Ident,
    field: &Field,
    trait_values: &mut Vec<TraitValue>,
) -> CompileResult {
    let mut ignore_ident: bool = true;

    for attr in &field.attrs {
        if attr.path.is_ident("cef_values") {
            return Err(
                SynError::new(attr.span(), CEF_VALUES_APPLICABLE.to_owned()).to_compile_error()
            );
        } else if attr.path.is_ident("cef_inherit") || attr.path.is_ident("cef_field") {
            let (message, value_type) = match attr.path.is_ident("cef_inherit") {
                true => (CEF_INHERIT_USAGE.to_owned(), FieldValueType::InheritTrait),
                false => (CEF_FIELD_USAGE.to_owned(), FieldValueType::DisplayTrait),
            };

            match parse_attrs_to_path(&attr, &message) {
                Err(e) => return Err(e),
                Ok(paths) => {
                    for p in paths {
                        if p.is_ident(header_name) {
                            let ts = field_value(
                                header_name,
                                method_name,
                                &value_type,
                                fieldid,
                                PrefixSelf::No,
                            );

                            // no longer ignore the ident
                            ignore_ident = false;

                            trait_values.push(TraitValue { ts, span: p.span() });
                        }
                    }
                }
            }
        }
    }

    match ignore_ident {
        true => match &field.ident {
            Some(ident) => Ok(quote! {#ident: _}),
            None => Ok(quote! {_}),
        },
        false => Ok(quote! {#fieldid}),
    }
}

// Helps cut through a lot of parse tree and doesn't confuse reading-context
fn parse_attrs_to_path(attr: &Attribute, messsage: &str) -> ParseAttrResult<Vec<Path>> {
    let mut paths: Vec<Path> = vec![];

    match attr.parse_meta() {
        Ok(Meta::List(list)) => {
            for nested_meta in list.nested {
                match nested_meta {
                    NestedMeta::Meta(Meta::Path(p)) => {
                        paths.push(p);
                    }
                    _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
                }
            }
        }
        Ok(_) => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
        Err(e) => return Err(e.to_compile_error()),
    }

    Ok(paths)
}

/// Generates a value from a field
fn field_value<T: quote::ToTokens>(
    header_name: &Ident,
    method_name: &Ident,
    value_type: &FieldValueType,
    field_name: T,
    prefix_self: PrefixSelf,
) -> TokenStream2 {
    let maybe_self = match prefix_self {
        PrefixSelf::Yes => quote! {&self.},
        PrefixSelf::No => quote! {},
    };

    match value_type {
        FieldValueType::InheritTrait => quote! {
            rust_cef::#header_name::#method_name(#maybe_self#field_name)
        },
        FieldValueType::DisplayTrait => quote! {
            Ok(format!("{}", #maybe_self#field_name))
        },
    }
}

/// Looks for the #[cef_values] attribute at the top level of a Struct,
/// Enum or Enum::Variant and returns a fixed string value.
///
/// For example:
///
/// ```ignore
/// #[derive(CefHeaderName)]
/// #[cef_values(CefHeaderName = "fixedName")]
/// struct Foo {
/// }
/// ```
///
/// ```ignore
/// #[derive(CefHeaderName)]
/// #[cef_values(CefHeaderName = "fixedName")]
/// enum Foo {
///     Variant1,
///     Variant2,
/// }
/// ```
///
/// ```ignore
/// #[derive(CefHeaderName)]
/// enum Foo {
///     #[cef_values(CefHeaderName = "fixedVariant1Name")]
///     Variant1,
///     #[cef_values(CefHeaderName = "fixedVariant2Name")]
///     Variant2,
/// }
/// ```
///
fn top_level_cef_values(
    header_name: &Ident,
    attrs: &[Attribute],
    trait_values: &mut Vec<TraitValue>,
) -> Option<TokenStream2> {
    for attr in attrs {
        if attr.path.is_ident("cef_inherit") {
            return Some(
                SynError::new(attr.path.span(), CEF_INHERIT_APPLICABLE.to_owned())
                    .to_compile_error(),
            );
        } else if attr.path.is_ident("cef_field") {
            return Some(
                SynError::new(attr.path.span(), CEF_FIELD_APPLICABLE.to_owned()).to_compile_error(),
            );
        } else if attr.path.is_ident("cef_values") {
            match parse_attrs_to_name_value(attr, &CEF_VALUES_USAGE) {
                Err(ts) => return Some(ts),
                Ok(mnvs) => {
                    for mnv in mnvs {
                        if mnv.path.is_ident(header_name) {
                            match &mnv.lit {
                                Lit::Str(strval) => {
                                    let ts = quote! {
                                        Ok(#strval.to_owned())
                                    };
                                    let span = mnv.span();
                                    trait_values.push(TraitValue { ts, span });
                                }
                                _ => {
                                    return Some(
                                        SynError::new(
                                            mnv.lit.span(),
                                            CEF_VALUES_STRINGS.to_owned(),
                                        )
                                        .to_compile_error(),
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
