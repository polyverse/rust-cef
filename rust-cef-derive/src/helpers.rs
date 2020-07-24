use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use std::convert::From;
use syn::{Data, DeriveInput, Error as SynError};

pub const CEF_ATTRIBUTE_APPLICATION: &str = "This attribute only applies to Structs or Enums.";

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
