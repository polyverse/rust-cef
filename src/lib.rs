use std::error::Error;

type CefResult<T> = Result<String, T>;

pub trait ToCef<T: Error> {
    fn to_cef(&self) -> CefResult<T>;
}

/********************************************************************************************** */
/* Tests! Tests! Tests! */

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::{Display, Formatter, Result as FmtResult};

    #[derive(Debug)]
    enum CefTestError {
        ExampleCase,
    }
    impl Error for CefTestError {}
    impl Display for CefTestError {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            write!(f, "CefTestError: {}", self)
        }
    }

    struct Example {}

    impl ToCef<CefTestError> for Example {
        fn to_cef(&self) -> CefResult<CefTestError> {
            Err(CefTestError::ExampleCase)
        }
    }

    #[test]
    fn test_impl() {
        let example = Example {};
        let result = example.to_cef();
        assert!(result.is_err())
    }
}
