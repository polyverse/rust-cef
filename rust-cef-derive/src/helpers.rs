use crate::proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::convert::From;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Error as SynError,
    Meta, NestedMeta, MetaNameValue,
};

pub const CEF_ATTRIBUTE_APPLICATION: &str = "This attribute only applies to Structs or Enums.";

pub type ParseAttrResult<T> = Result<T, TokenStream2>;

pub fn is_valid_item_type(item: &DeriveInput) -> Option<TokenStream> {
    // Only applies to structs and enums
    match item.data {
        Data::Struct(_) | Data::Enum(_) => {}
        _ => {
            return Some(TokenStream::from(
                SynError::new(Span::call_site(), CEF_ATTRIBUTE_APPLICATION).to_compile_error(),
            ))
        }
    }

    None
}

// Helps cut through a lot of parse tree and doesn't confuse reading-context
pub fn parse_attrs_to_name_value(
    attr: &Attribute,
    message: &str,
) -> ParseAttrResult<Vec<MetaNameValue>> {
    let mut mnvs: Vec<MetaNameValue> = vec![];

    match attr.parse_meta() {
        Err(e) => return Err(e.to_compile_error()),
        Ok(Meta::List(list)) => {
            for nestedmeta in list.nested {
                match nestedmeta {
                    NestedMeta::Meta(Meta::NameValue(mnv)) => {
                        mnvs.push(mnv);
                    },
                    _ => return Err(SynError::new(attr.span(), message.to_owned()).to_compile_error()),
                }
            }
        }
        Ok(_) => return Err(SynError::new(attr.span(), message.to_owned()).to_compile_error()),
    }

    Ok(mnvs)
}
