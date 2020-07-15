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
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error as SynError, Field, Fields, Ident,
    Lit, Meta, MetaNameValue, NestedMeta, Path, Variant,
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

type ParseAttrResult<T> = Result<T, TokenStream2>;

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
#[proc_macro_derive(
    CefHeaderDeviceEventClassID,
    attributes(cef_values, cef_inherit, cef_field)
)]
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

/// Implements the trait asked by any of the `#[derive(CefHeader*)]` attributes
/// It creates the trait skeleton and outsources the returned value
/// to a child-item function.
fn implement_trait(trait_name_str: &str, item: &DeriveInput) -> TokenStream {
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
        Data::Union(_) => {
            return SynError::new(Span::call_site(), CEF_HEADERS_APPLICATION.to_owned())
                .to_compile_error()
        }
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

                match parse_attrs_to_path(attr, usage_message.as_str()) {
                    Ok(paths) => {
                        for p in paths {
                            if p.is_ident(header_name) {
                                let field_name = match &field.ident {
                                    Some(i) => format_ident!("{}", i),
                                    None => format_ident!("{}", index),
                                };

                                // let's use our field's value.. either through format! or the trait
                                let ts = match attr.path.is_ident("cef_inherit") {
                                    true => quote! {
                                        rust_cef::#header_name::#method_name(&self.#field_name)
                                    },
                                    false => quote! {
                                        Ok(format!("{}", &self.#field_name))
                                    },
                                };

                                let span = p.span().clone();

                                let tv = TraitValue { ts, span };

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

fn top_level_cef_values(
    header_name: &Ident,
    item: &DeriveInput,
    trait_values: &mut Vec<TraitValue>,
) -> Option<TokenStream2> {
    for attr in &item.attrs {
        if attr.path.is_ident("cef_inherit") {
            return Some(
                SynError::new(attr.path.span(), CEF_INHERIT_STRUCT_USAGE.to_owned())
                    .to_compile_error(),
            );
        }

        if attr.path.is_ident("cef_values") {
            match parse_attrs_to_name_value(attr, &CEF_VALUES_STRUCT_USAGE) {
                Err(ts) => return Some(ts),
                Ok(mnvs) => {
                    for mnv in mnvs {
                        if mnv.path.is_ident(header_name) {
                            match &mnv.lit {
                                Lit::Str(strval) => {
                                    let ts = quote! {
                                        Ok(#strval.to_owned())
                                    };
                                    let span = mnv.span().clone();
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
/// `#[cef_values(HeaderName = value)]`
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
/// `#[cef_inherit(HeaderName = ["field" | index])`
/// This attribute applies only to a variant, and sets the Header value
/// inherited from a field (when variant has named fields) or an index (
/// when variant has unnamed fields.)
///
/// Essentially, this looks like
/// ```ignore
/// #[derive(Header)]
/// enum Items {
///     #[cef_inherit(Header = 1)]
///     Variant1(int, HeaderImplementer1),
///     #[cef_inherit(Header = "address")]
///     Variant2{name: String, address: HeaderImplementer2},
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
///     #[cef_field(Header = 1)]
///     Variant1(int, DisplayImplementer1),
///     #[cef_field(Header = "address")]
///     Variant2{name: String, address: DisplayImplementer2},
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
///     #[cef_field(Header = 1)]
///     Variant1(int, DisplayImplementer1),
///     #[cef_inherit(Header = "address")]
///     Variant2{name: String, address: HeaderImplementer2},
///     #[cef_value(Header = "variant3")]
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
    if let Some(ts) = top_level_cef_values(header_name, item, &mut trait_values) {
        return ts;
    }

    // Set CEF value for this header from every variant
    if let Some(ts) =
        all_variants_cef_value(header_name, method_name, &item.ident, &e, &mut trait_values)
    {
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

/// This function creates a match statement with args for every variant for the Enum
/// this is what allows a unified Header trait to be implemented on the Enum.
///
fn all_variants_cef_value(
    header_name: &Ident,
    method_name: &Ident,
    enum_name: &Ident,
    e: &DataEnum,
    trait_values: &mut Vec<TraitValue>,
) -> Option<TokenStream2> {
    let mut variant_values: Vec<(Variant, TokenStream2)> = vec![];

    // now look for struct's variants and get the header value for either ALL variants or NO variants
    for variant in e.variants.iter() {
        // Generate values for each variant in the enum (so that it can be branched later)
        if let Some(ts) = variant_value(header_name, method_name, variant, &mut variant_values) {
            return Some(ts);
        }
    }

    if variant_values.len() > 0 && variant_values.len() < e.variants.len() {
        return Some(SynError::new(Span::call_site(), format!("For this enum, not all variants implement the trait {}. For Enum traits, each and every variant must supply a value, either through 'cef_values' or 'cef_inherit' macros.", header_name)).to_compile_error());
    }

    if variant_values.len() == e.variants.len() {
        let match_branches: Vec<TokenStream2> = variant_values
            .iter()
            .map(|(variant, val)| {
                // Get the identity of the Variant
                let ident = variant.ident.clone();

                // Create the fields for the Variant
                // Specifically this part:
                //
                // ```
                // V1(_ident0, ident1, ..)
                //   ^^^^^^^^^^^^^^^^^
                // V2{name: String, address: String}
                //   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //```
                //
                // So the match can be used later
                let variant_field_match_expr = match &variant.fields {
                    Fields::Named(namedf) => {
                        // create this field-matcher for named fields
                        // {name: _name, address: _address}
                        // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                        let named_fields: Vec<TokenStream2> = namedf
                            .named
                            .iter()
                            .map(|f| -> TokenStream2 {
                                // this may panic - we're under named fields. Can't coddle everyone.
                                let id = f.ident.as_ref().unwrap();
                                let ignore_id = format_ident!("_{}", id);
                                return quote! {#id: #ignore_id};
                            })
                            .collect();

                        // enclose named fields in {}
                        quote! {
                            {#(#named_fields),*}
                        }
                    }
                    Fields::Unnamed(unnamedf) => {
                        // Create this field matcher for unnamed fields
                        // (_ident0, ident1, ..)
                        // ^^^^^^^^^^^^^^^^^
                        let unnamed_fields: Vec<TokenStream2> = unnamedf
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(idx, _)| -> TokenStream2 {
                                let ident = format_ident!("_ident{}", idx);
                                quote! {#ident}
                            })
                            .collect();

                        // enclose unnamed fields in ()
                        quote! {
                            (#(#unnamed_fields),*)
                        }
                    }
                    // When no fields, generate nothing
                    Fields::Unit => quote! {},
                };

                // The overall match branch now looks like this:
                // Items::Variant1{name: _name, address: _address}  => format!("{}", _name)
                // ^^^^^  ^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^      ^^^^^^^^^^^^^^^^^^^^
                // enum   variant   field matchers we just created     The value for this variant (which may refer to the fields - even unnamed fields are captured under positional names _ident0, _ident1, etc.)
                let match_branch = quote! {
                    #enum_name::#ident#variant_field_match_expr => #val,
                };

                match_branch
            })
            .collect();

        // Finall match all branches
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
    }

    None
}

/// Generate a value for a variant in the enum (so that it can be branched later)
/// The values can use well-known field names for named-fields and well-known positional
/// names for unnamed fields
///
/// For named fields, the capture name is _<fieldname>
/// For example:
/// enum Items {
///     V1{name: String, address: String}
/// }
///
/// would be captured as:
/// match enumVariant {
///     V1{name: _name, address: _address} => // value can use _name or _address
/// }
///
/// For unnamed fields, the capture name is _ident<index>
/// For example:
/// enum Items {
///     V1(String, String}
/// }
///
/// would be captured as:
/// match enumVariant {
///     V1(_ident0, _ident1) => // value can use _ident0 or _ident1
/// }
///
fn variant_value(
    header_name: &Ident,
    method_name: &Ident,
    variant: &Variant,
    variant_values: &mut Vec<(Variant, TokenStream2)>,
) -> Option<TokenStream2> {
    let mut in_variant_values: Vec<TraitValue> = vec![];

    // find values from attributes
    for attr in &variant.attrs {
        // primarily one of these three attributes
        if attr.path.is_ident("cef_inherit")
            || attr.path.is_ident("cef_values")
            || attr.path.is_ident("cef_field")
        {
            let usage_message = match attr.path.is_ident("cef_inherit") {
                true => CEF_INHERIT_ENUM_USAGE.to_owned(),
                false => match attr.path.is_ident("cef_values") {
                    true => CEF_VALUES_ENUM_USAGE.to_owned(),
                    false => CEF_FIELD_ENUM_USAGE.to_owned(),
                },
            };

            match parse_attrs_to_name_value(attr, usage_message.as_str()) {
                Ok(mnvs) => {
                    for nv in mnvs {
                        if nv.path.is_ident(header_name) {
                            // we found the attribute for this header
                            if let Some(ts) = variant_value_from_attribute(
                                header_name,
                                method_name,
                                attr,
                                nv,
                                &variant.fields,
                                &mut in_variant_values,
                            ) {
                                return Some(ts);
                            }
                        }
                    }
                }
                Err(e) => return Some(e),
            }
        }
    }

    if in_variant_values.len() > 1 {
        let errs = in_variant_values.iter().map(|tv| {
            return SynError::new(tv.span, format!("Trait {} had values provided in multiple places for variant {}. Please remove all but one of these.", header_name, variant.ident))
                .to_compile_error();
        });

        return Some(quote! {
            #(#errs)*
        });
    } else if in_variant_values.len() == 1 {
        match in_variant_values.pop() {
            Some(tv) => {variant_values.push((variant.clone(), tv.ts));},
            None => return Some(SynError::new(Span::call_site(), "FATAL Error in this macro. Thought it generated a value, but it apparently did not.".to_owned()).to_compile_error()),
        }
    }

    None
}

/// Generates a value for Header on an Enum variant, based on the attribute being parsed
fn variant_value_from_attribute(
    header_name: &Ident,
    method_name: &Ident,
    attr: &Attribute,
    nv: MetaNameValue,
    fields: &Fields,
    in_variant_values: &mut Vec<TraitValue>,
) -> Option<TokenStream2> {
    // First it looks at whether the attribute specifies a field by position or index
    let ts = match &nv.lit {
        Lit::Int(litint) => match litint.base10_parse::<usize>() {
            // if positional, let's find the n'th index if one exists
            Ok(i) => match fields.iter().nth(i) {
                //let's find the field index - only if necessary
                Some(field) => match &field.ident {
                    Some(ident) => return Some(SynError::new(nv.span(), format!("Field at index {} has a name '{}'. Don't use indexes for when not needed..", i, ident)).to_compile_error()),
                    None => {
                        // since the indexed field truly has no ident, we generate a well-known accessor
                        let field_index = format_ident!("_ident{}", i);

                        match attr.path.is_ident("cef_inherit") {
                            // are we inheriting
                            true => quote! {
                                Ok(rust_cef::#header_name::#method_name(&#field_index))
                            },
                            false => match attr.path.is_ident("cef_field") {
                                // or field fmt::Display'ing?
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
            // if named, is there a direct value to set?
            true => quote! { Ok(#s.to_owned())},
            false => {
                // no direct value, so we must inherit or format!
                // First let's find our fieldid, and then turn it into our well-known capture:
                // _fieldid
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

                // Now pick a value that's either format! or inherited
                match attr.path.is_ident("cef_field") {
                    true => quote! {
                            Ok(format!("{}", #fieldid).to_owned())
                        },
                    false =>  quote! {
                        rust_cef::#header_name::#method_name(#fieldid)
                    },
                }
            },
        },
        _ => return Some(SynError::new(nv.lit.span(), format!("Value for {} can be a string literal (for direct placement), index, or field name.", header_name)).to_compile_error()),
    };

    let span = nv.span().clone();

    let tv = TraitValue { ts, span };

    in_variant_values.push(tv);

    None
}

// Helps cut through a lot of parse tree and doesn't confuse reading-context
fn parse_attrs_to_path(attr: &Attribute, messsage: &str) -> ParseAttrResult<Vec<Path>> {
    let mut paths: Vec<Path> = vec![];

    match attr.parse_meta() {
        Ok(parsed_meta) => match parsed_meta {
            Meta::List(list) => {
                for nested_meta in list.nested {
                    match nested_meta {
                        NestedMeta::Meta(meta) => match meta {
                            Meta::Path(p) => {
                                paths.push(p);
                            }
                            _ => {
                                return Err(SynError::new(attr.span(), messsage).to_compile_error())
                            }
                        },
                        _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
                    }
                }
            }
            _ => return Err(SynError::new(attr.span(), messsage).to_compile_error()),
        },
        Err(e) => return Err(e.to_compile_error()),
    }

    Ok(paths)
}
// Helps cut through a lot of parse tree and doesn't confuse reading-context
fn parse_attrs_to_name_value(
    attr: &Attribute,
    message: &str,
) -> ParseAttrResult<Vec<MetaNameValue>> {
    let mut mnvs: Vec<MetaNameValue> = vec![];

    match attr.parse_meta() {
        Err(e) => return Err(e.to_compile_error()),
        Ok(metadata) => match metadata {
            Meta::List(list) => {
                for nestedmeta in list.nested {
                    match nestedmeta {
                        NestedMeta::Meta(meta) => match meta {
                            Meta::NameValue(mnv) => {
                                mnvs.push(mnv);
                            }
                            _ => {
                                return Err(SynError::new(attr.span(), message.to_owned())
                                    .to_compile_error())
                            }
                        },
                        _ => {
                            return Err(
                                SynError::new(attr.span(), message.to_owned()).to_compile_error()
                            )
                        }
                    }
                }
            }
            _ => return Err(SynError::new(attr.span(), message.to_owned()).to_compile_error()),
        },
    }

    Ok(mnvs)
}
