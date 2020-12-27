use std::collections::HashMap;
/// Copyright 2020 Polyverse Corporation
/// This module provides traits to allow arbitrary Rust items (structs, enums, etc.)
/// to be converted into Common Event Format strings used by popular loggers around the world.
///
/// This is primarily built to have guard rails and ensure the CEF doesn't
/// break by accident when making changes to Rust items.
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use time::OffsetDateTime;

/// An error consistently used all code
/// in this module and sub-modules.
///
/// May have structured errors, and arbitrary errors
/// are flagged as `Unexpected(s)` with the string `s`
/// containing the message.
///
#[derive(Debug, PartialEq)]
pub enum CefConversionError {
    Unexpected(String),
}
impl Error for CefConversionError {}
impl Display for CefConversionError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            CefConversionError::Unexpected(message) => {
                write!(f, "CefConversionError::Unexpected {}", message)
            }
        }
    }
}

/// CefResult is the consistent result type used by all
/// code in this module and sub-modules
pub type CefResult = Result<String, CefConversionError>;

// CefExtensionsResult is used to return an error when necessary
// but nothing useful when it works. Making it an error
// provides proper context vs doing Option
pub type CefExtensionsResult = Result<(), CefConversionError>;

/// A trait that returns the "Version" CEF Header
pub trait CefHeaderVersion {
    fn cef_header_version(&self) -> CefResult;
}

/// A trait that returns the "DeviceVendor" CEF Header
pub trait CefHeaderDeviceVendor {
    fn cef_header_device_vendor(&self) -> CefResult;
}

/// A trait that returns the "DeviceProduct" CEF Header
pub trait CefHeaderDeviceProduct {
    fn cef_header_device_product(&self) -> CefResult;
}

/// A trait that returns the "DeviceVersion" CEF Header
pub trait CefHeaderDeviceVersion {
    fn cef_header_device_version(&self) -> CefResult;
}

/// A trait that returns the "DeviceEventClassID" CEF Header
pub trait CefHeaderDeviceEventClassID {
    fn cef_header_device_event_class_id(&self) -> CefResult;
}

/// A trait that returns the "Name" CEF Header
pub trait CefHeaderName {
    fn cef_header_name(&self) -> CefResult;
}

/// A trait that returns the "Severity" CEF Header
pub trait CefHeaderSeverity {
    fn cef_header_severity(&self) -> CefResult;
}

/// A trait that returns CEF Extensions. This is a roll-up
/// trait that should ideally take into account any CEF extensions
/// added by sub-fields or sub-objects from the object on which
/// this is implemented.
pub trait CefExtensions {
    fn cef_extensions(&self, collector: &mut HashMap<String, String>) -> CefExtensionsResult;
}

/// This trait emits an ArcSight Common Event Format
/// string by combining all the other traits that provide
/// CEF headers and extensions.
pub trait ToCef:
    CefHeaderVersion
    + CefHeaderDeviceVendor
    + CefHeaderDeviceProduct
    + CefHeaderDeviceVersion
    + CefHeaderDeviceEventClassID
    + CefHeaderName
    + CefHeaderSeverity
    + CefExtensions
{
    fn to_cef(&self) -> CefResult {
        let mut extensions: HashMap<String, String> = HashMap::new();

        // get our extensions
        if let Err(err) = self.cef_extensions(&mut extensions) {
            return Err(err);
        };

        // make it into key=value strings
        let mut kvstrs: Vec<String> = extensions
            .into_iter()
            .map(|(key, value)| [key, value].join("="))
            .collect();

        kvstrs.sort_unstable();

        // Make it into a "key1=value1 key2=value2" string (each key=value string concatenated and separated by spaces)
        let extensionsstr = kvstrs.join(" ");

        let mut cef_entry = String::new();
        cef_entry.push_str("CEF:");
        cef_entry.push_str(&self.cef_header_version()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_device_vendor()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_device_product()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_device_version()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_device_event_class_id()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_name()?);
        cef_entry.push('|');
        cef_entry.push_str(&self.cef_header_severity()?);
        cef_entry.push('|');
        cef_entry.push_str(extensionsstr.as_str());

        Ok(cef_entry)
    }
}

/// Implement CefExtensions (since it's defined here) for type
/// DateTime<Utc>
impl CefExtensions for OffsetDateTime {
    /// we serialize using:
    /// Milliseconds since January 1, 1970 (integer). (This time format supplies an integer
    /// with the count in milliseconds from January 1, 1970 to the time the event occurred.)
    fn cef_extensions(&self, collector: &mut HashMap<String, String>) -> CefExtensionsResult {
        collector.insert(
            "rt".to_owned(),
            format!("{}", self.unix_timestamp_nanos() / 1000000),
        );
        Ok(())
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
        fn cef_extensions(&self, collector: &mut HashMap<String, String>) -> CefExtensionsResult {
            collector.insert("customField1".to_owned(), "customValue1".to_owned());
            collector.insert("customField2".to_owned(), "customValue2".to_owned());
            collector.insert("customField3".to_owned(), "customValue2".to_owned());
            collector.insert("customField4".to_owned(), "customValue3".to_owned());
            Ok(())
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
            Err(CefConversionError::Unexpected(
                "This error should propagate".to_owned(),
            ))
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
        fn cef_extensions(&self, collector: &mut HashMap<String, String>) -> CefExtensionsResult {
            collector.insert("customField".to_owned(), "customValue".to_owned());
            Ok(())
        }
    }

    #[test]
    fn test_impl_works() {
        let example = GoodExample {};
        let result = example.to_cef();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CEF:0|polyverse|zerotect|V1|LinuxKernelTrap|Linux Kernel Trap|10|customField1=customValue1 customField2=customValue2 customField3=customValue2 customField4=customValue3");
    }

    #[test]
    fn test_error_propagates() {
        let example = BadExample {};
        let result = example.to_cef();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CefConversionError::Unexpected("This error should propagate".to_owned())
        );
    }

    #[test]
    fn test_ext_for_datetime() {
        let mut collector = HashMap::<String, String>::new();
        let example = OffsetDateTime::from_unix_timestamp_nanos(3435315515325000000);
        let result = example.cef_extensions(&mut collector);
        assert!(result.is_ok());

        let maybe_rt = collector.get("rt");
        assert!(maybe_rt.is_some());

        let rt = maybe_rt.unwrap();
        assert_eq!(rt, "3435315515325");
    }
}
