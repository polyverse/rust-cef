use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, PartialEq)]
pub enum CefConversionError {
    Unexpected(String),
}
impl Error for CefConversionError {}
impl Display for CefConversionError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            CefConversionError::Unexpected(message) => write!(f, "CefConversionError::Unexpected {}", message),
        }
    }
}

pub type CefResult = Result<String, CefConversionError>;

pub trait ToCef {
    fn to_cef(&self) -> CefResult;
}

/********************************************************************************************** */
/* Tests! Tests! Tests! */

#[cfg(test)]
mod test {
    use super::*;
    struct Example {}

    impl ToCef for Example {
        fn to_cef(&self) -> CefResult {
            Err(CefConversionError::Unexpected("Hope this works".to_owned()))
        }
    }

    #[test]
    fn test_impl() {
        let example = Example {};
        let result = example.to_cef();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CefConversionError::Unexpected("Hope this works".to_owned()));
    }
}
