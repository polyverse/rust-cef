#[macro_use]
extern crate rust_cef_derive;

use rust_cef::{CefExtensions, CefHeaderName, CefHeaderVersion, ToCef};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[test]
fn test_cef_fixed_headers_fails() {
    let _t = trybuild::TestCases::new();
}

#[test]
fn test_cef_fixed_headers() {
    let sh = SingleHeader {};
    assert_eq!(sh.cef_header_version().unwrap(), "4234");

    let ma = MultipleAttrs {};
    assert_eq!(ma.cef_header_version().unwrap(), "3424");
    assert_eq!(ma.cef_header_name().unwrap(), "name1");

    let mh = MultipleHeaders {};
    assert_eq!(mh.cef_header_version().unwrap(), "3235");
    assert_eq!(mh.cef_header_name().unwrap(), "name2");
}

#[test]
fn test_to_cef_with_fixed_headers_and_custom_extensions() {
    let t = AllFixedHeadersCustomExtensions {};
    assert_eq!(
        t.to_cef().unwrap(),
        "CEF:0|polyverse|zerotect|V1|LinuxKernelFault|Linux Kernel Fault|10|extension1=value1"
    )
}

#[test]
fn test_to_cef_with_fixed_and_manual_headers() {
    let t = ManualAndFixedHeaders {};
    assert_eq!(
        t.to_cef().unwrap(),
        "CEF:customVersion|polyverse|zerotect|V1|LinuxKernelFault|Linux Kernel Fault|10|"
    )
}

#[test]
fn test_cef_extensions() {
    let n1 = NameStruct {
        name: "WillBeRenamed".to_owned(),
    };
    assert_eq!(n1.cef_extensions().unwrap(), "newname=WillBeRenamed");
    // Header implementation still works
    assert_eq!(n1.cef_header_name().unwrap(), "WillBeRenamed");

    let n2 = NameInheritorStruct {
        name_struct: NameStruct {
            name: "NS1".to_owned(),
        },
        name_struct2: NameStruct {
            name: "NS2".to_owned(),
        },
        address: "An address of some sort".to_owned(),
        age: 42,
    };
    assert_eq!(
        n2.cef_extensions().unwrap(),
        "newname=NS1 address=An address of some sort name2=NameStruct::NS2 person_age=42"
    );
}

#[test]
fn test_complete_to_cef() {
    let v1 = Top::V1(
        "ClassId234".to_owned(),
        NameInheritorStruct {
            name_struct: NameStruct {
                name: "Test1".to_owned(),
            },
            name_struct2: NameStruct {
                name: "Test2".to_owned(),
            },
            address: "Address".to_owned(),
            age: 87,
        },
        24,
    );
    assert_eq!(
        v1.to_cef().unwrap(),
        "CEF:1|polyverse|zerotect|V1|ClassId234|NameInheritorStruct::NameStruct::Test1|24|"
    );

    let v2 = Top::V2 {
        event_class: "ClassId234",
        name_impl: NameInheritorStruct {
            name_struct: NameStruct {
                name: "Test2".to_owned(),
            },
            name_struct2: NameStruct {
                name: "Test1".to_owned(),
            },
            address: "Address2".to_owned(),
            age: 78,
        },
        severity: 85,
    };

    assert_eq!(
        v2.to_cef().unwrap(),
        "CEF:1|polyverse|zerotect|V2|ClassId234|Test2|85|"
    );
}

/**************************** Test Structs ******************************************/

#[derive(CefHeaderVersion, CefHeaderName)]
#[cef_values(CefHeaderVersion = "3235", CefHeaderName = "name2")]
struct MultipleHeaders {}

#[derive(CefHeaderVersion)]
#[cef_values(CefHeaderVersion = "3424")]
#[cef_values(CefHeaderName = "name1")]
#[derive(CefHeaderName)]
struct MultipleAttrs {}

#[cef_values(CefHeaderVersion = "4234")]
#[derive(CefHeaderVersion)]
struct SingleHeader {}

#[derive(
    CefHeaderVersion,
    CefHeaderDeviceVendor,
    CefHeaderDeviceProduct,
    CefHeaderDeviceVersion,
    CefHeaderDeviceEventClassID,
    CefHeaderName,
    CefHeaderSeverity,
)]
#[cef_values(
    CefHeaderVersion = "0",
    CefHeaderDeviceVendor = "polyverse",
    CefHeaderDeviceProduct = "zerotect",
    CefHeaderDeviceVersion = "V1",
    CefHeaderDeviceEventClassID = "LinuxKernelFault",
    CefHeaderName = "Linux Kernel Fault",
    CefHeaderSeverity = "10"
)]
#[derive(ToCef)]
struct AllFixedHeadersCustomExtensions {}
impl CefExtensions for AllFixedHeadersCustomExtensions {
    fn cef_extensions(&self) -> rust_cef::CefResult {
        Ok("extension1=value1".to_owned())
    }
}

#[derive(
    CefHeaderDeviceVendor,
    CefHeaderDeviceProduct,
    CefHeaderDeviceVersion,
    CefHeaderDeviceEventClassID,
    CefHeaderName,
    CefHeaderSeverity,
)]
#[cef_values(
    CefHeaderDeviceVendor = "polyverse",
    CefHeaderDeviceProduct = "zerotect"
)]
#[cef_values(
    CefHeaderName = "Linux Kernel Fault",
    CefHeaderSeverity = "10",
    CefHeaderDeviceVersion = "V1",
    CefHeaderDeviceEventClassID = "LinuxKernelFault"
)]
#[derive(ToCef, CefExtensions)]
struct ManualAndFixedHeaders {}
impl CefHeaderVersion for ManualAndFixedHeaders {
    fn cef_header_version(&self) -> rust_cef::CefResult {
        Ok("customVersion".to_owned())
    }
}

#[derive(
    CefHeaderVersion,
    CefHeaderDeviceVendor,
    CefHeaderDeviceVersion,
    CefHeaderDeviceEventClassID,
    CefHeaderName,
    CefHeaderDeviceProduct,
    CefHeaderSeverity,
)]
#[cef_values(
    CefHeaderVersion = "1",
    CefHeaderDeviceVendor = "polyverse",
    CefHeaderDeviceProduct = "zerotect"
)]
#[derive(ToCef)]
enum Top {
    // Name will use the display trait, rather than inheriting the CefHeaderName trait
    #[cef_values(CefHeaderDeviceVersion = "V1")]
    V1(
        #[cef_field(CefHeaderDeviceEventClassID)] String,
        #[cef_field(CefHeaderName)] NameInheritorStruct,
        #[cef_field(CefHeaderSeverity)] usize,
    ),

    #[cef_values(CefHeaderDeviceVersion = "V2")]
    V2 {
        #[cef_field(CefHeaderDeviceEventClassID)]
        event_class: &'static str,
        #[cef_inherit(CefHeaderName)]
        name_impl: NameInheritorStruct,
        #[cef_field(CefHeaderSeverity)]
        severity: usize,
    },
}

impl CefExtensions for Top {
    fn cef_extensions(&self) -> rust_cef::CefResult {
        Ok("".to_owned())
    }
}

#[derive(CefHeaderName)]
struct TupleStule(#[cef_inherit(CefHeaderName)] NameStruct);

#[derive(CefHeaderName, CefExtensions)]
struct NameInheritorStruct {
    // using
    // #[cef_ext_field]
    // would do: name_struct.to_string()
    // but we want to gobble extension field's created inside NameStruct
    #[cef_ext_gobble]
    #[cef_inherit(CefHeaderName)]
    pub name_struct: NameStruct,

    #[cef_ext_field]
    pub address: String,

    #[cef_ext_field(name2)]
    pub name_struct2: NameStruct,

    #[cef_ext_field(person_age)]
    pub age: usize,
}

impl Display for NameInheritorStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NameInheritorStruct::{}", self.name_struct)
    }
}

#[derive(CefHeaderName, CefExtensions)]
struct NameStruct {
    // use the field's name
    #[cef_ext_field(newname)]
    #[cef_field(CefHeaderName)]
    pub name: String,
}

impl Display for NameStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NameStruct::{}", self.name)
    }
}
