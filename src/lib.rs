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

pub trait CefHeaderVersion {
    fn cef_header_version(&self) -> CefResult;
}

pub trait CefHeaderDeviceVendor {
    fn cef_header_device_vendor(&self) -> CefResult;
}

pub trait CefHeaderDeviceProduct {
    fn cef_header_device_product(&self) -> CefResult;
}

pub trait CefHeaderDeviceVersion {
    fn cef_header_device_version(&self) -> CefResult;
}

pub trait CefHeaderDeviceEventClassID {
    fn cef_header_device_event_class_id(&self) -> CefResult;
}

pub trait CefHeaderName {
    fn cef_header_name(&self) -> CefResult;
}

pub trait CefHeaderSeverity {
    fn cef_header_severity(&self) -> CefResult;
}

pub trait CefExtensions {
    fn cef_extensions(&self) -> CefResult;
}

pub trait ToCef :
        CefHeaderVersion +
        CefHeaderDeviceVendor +
        CefHeaderDeviceProduct +
        CefHeaderDeviceVersion +
        CefHeaderDeviceEventClassID +
        CefHeaderName +
        CefHeaderSeverity +
        CefExtensions {

    fn to_cef(&self) -> CefResult {
        let mut cef_entry = String::new();
        cef_entry.push_str("CEF:");
        cef_entry.push_str(&self.cef_header_version()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_device_vendor()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_device_product()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_device_version()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_device_event_class_id()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_name()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_header_severity()?);
        cef_entry.push_str("|");
        cef_entry.push_str(&self.cef_extensions()?);

        Ok(cef_entry)
    }
}


/********************************************************************************************** */
/* Tests! Tests! Tests! */

#[cfg(test)]
mod test {
    use super::*;
    struct GoodExample {}

    impl ToCef for GoodExample {}
    impl CefHeaderVersion for GoodExample {
        fn cef_header_version(&self) -> CefResult {
            Ok("0".to_owned())
        }
    }

    impl CefHeaderDeviceVendor for GoodExample {
        fn cef_header_device_vendor(&self) -> CefResult {
            Ok("polyverse".to_owned())
        }
    }

    impl CefHeaderDeviceProduct for GoodExample {
        fn cef_header_device_product(&self) -> CefResult {
            Ok("zerotect".to_owned())
        }
    }

    impl CefHeaderDeviceVersion for GoodExample {
        fn cef_header_device_version(&self) -> CefResult {
            Ok("V1".to_owned())
        }
    }

    impl CefHeaderDeviceEventClassID for GoodExample {
        fn cef_header_device_event_class_id(&self) -> CefResult {
            Ok("LinuxKernelTrap".to_owned())
        }
    }

    impl CefHeaderName for GoodExample {
        fn cef_header_name(&self) -> CefResult {
            Ok("Linux Kernel Trap".to_owned())
        }
    }

    impl CefHeaderSeverity for GoodExample {
        fn cef_header_severity(&self) -> CefResult {
            Ok("10".to_owned())
        }
    }

    impl CefExtensions for GoodExample {
        fn cef_extensions(&self) -> CefResult {
            Ok("customField=customValue".to_owned())
        }
    }

    struct BadExample {}
    impl ToCef for BadExample {}
    impl CefHeaderVersion for BadExample {
        fn cef_header_version(&self) -> CefResult {
            Ok("0".to_owned())
        }
    }

    impl CefHeaderDeviceVendor for BadExample {
        fn cef_header_device_vendor(&self) -> CefResult {
            Ok("polyverse".to_owned())
        }
    }

    impl CefHeaderDeviceProduct for BadExample {
        fn cef_header_device_product(&self) -> CefResult {
            Ok("zerotect".to_owned())
        }
    }

    impl CefHeaderDeviceVersion for BadExample {
        fn cef_header_device_version(&self) -> CefResult {
            Ok("V1".to_owned())
        }
    }

    impl CefHeaderDeviceEventClassID for BadExample {
        fn cef_header_device_event_class_id(&self) -> CefResult {
            Err(CefConversionError::Unexpected("This error should propagate".to_owned()))
        }
    }

    impl CefHeaderName for BadExample {
        fn cef_header_name(&self) -> CefResult {
            Ok("Linux Kernel Trap".to_owned())
        }
    }

    impl CefHeaderSeverity for BadExample {
        fn cef_header_severity(&self) -> CefResult {
            Ok("10".to_owned())
        }
    }

    impl CefExtensions for BadExample {
        fn cef_extensions(&self) -> CefResult {
            Ok("customField=customValue".to_owned())
        }
    }


    #[test]
    fn test_impl_works() {
        let example = GoodExample {};
        let result = example.to_cef();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CEF:0|polyverse|zerotect|V1|LinuxKernelTrap|Linux Kernel Trap|10|customField=customValue");
    }

    #[test]
    fn test_error_propagates() {
        let example = BadExample {};
        let result = example.to_cef();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CefConversionError::Unexpected("This error should propagate".to_owned()));
    }
}
