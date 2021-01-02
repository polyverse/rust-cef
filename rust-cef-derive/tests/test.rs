use rust_cef_derive::{
    CefExtensions, CefHeaderDeviceEventClassID, CefHeaderDeviceProduct, CefHeaderDeviceVendor,
    CefHeaderDeviceVersion, CefHeaderName, CefHeaderSeverity, CefHeaderVersion, ToCef,
};

use rust_cef::{CefExtensions, CefHeaderName, CefHeaderVersion, ToCef};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use time::OffsetDateTime;

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
    let mut collector = HashMap::<String, String>::new();
    assert!(n1.cef_extensions(&mut collector).is_ok());
    assert_eq!(
        collector.get(&"newname".to_owned()),
        Some(&"WillBeRenamed".to_owned())
    );
    // Header implementation still works
    assert_eq!(n1.cef_header_name().unwrap(), "WillBeRenamed");

    let n2 = NameInheritorStruct {
        name_struct: NameStruct {
            name: "NS1".to_owned(),
        },
        name_struct2: Some(NameStruct {
            name: "NS2".to_owned(),
        }),
        address: Some("An address of some sort".to_owned()),
        age: 42,
    };

    let mut collector = HashMap::<String, String>::new();
    assert!(n2.cef_extensions(&mut collector).is_ok());
    assert_eq!(
        collector.get(&"newname".to_owned()),
        Some(&"NS2".to_owned())
    );
    assert_eq!(
        collector.get(&"address".to_owned()),
        Some(&"An address of some sort".to_owned())
    );
    assert_eq!(
        collector.get(&"name2".to_owned()),
        Some(&"NameStruct::NS1".to_owned())
    );
    assert_eq!(
        collector.get(&"person_age".to_owned()),
        Some(&"42".to_owned())
    );
}

#[test]
fn test_complete_to_cef() {
    let v1 = Top::V1(
        "ClassId234".to_owned(),
        NameInheritorStruct {
            name_struct: NameStruct {
                name: "Test2".to_owned(),
            },
            name_struct2: Some(NameStruct {
                name: "Test1".to_owned(),
            }),
            address: Some("Address".to_owned()),
            age: 87,
        },
        24,
        OffsetDateTime::from_unix_timestamp_nanos(735027350723000000),
    );
    assert_eq!(
        v1.to_cef().unwrap(),
        "CEF:1|polyverse|zerotect|V1|ClassId234|NameInheritorStruct::NameStruct::Test2|24|EnumV1Field=fixedExtensionsValue TopEnumField=fixedExtensionsValue TopStructField=fixedExtensionsValue address=Address name2=NameStruct::Test2 newname=Test1 person_age=87 rt=735027350723 top_name=ClassId234"
    );

    let v2 = Top::V2 {
        event_class: "ClassId234",
        name_impl: NameInheritorStruct {
            name_struct: NameStruct {
                name: "Test2".to_owned(),
            },
            name_struct2: Some(NameStruct {
                name: "Test1".to_owned(),
            }),
            address: Some("Address2".to_owned()),
            age: 78,
        },
        severity: 85,
        unused: 20,
        timestamp: OffsetDateTime::from_unix_timestamp_nanos(326262362000000),
    };

    assert_eq!(
        v2.to_cef().unwrap(),
        "CEF:1|polyverse|zerotect|V2|ClassId234|Test2|85|EnumV2Field=fixedExtensionsValue EventClassNewName=ClassId234 TopEnumField=fixedExtensionsValue TopStructField=fixedExtensionsValue address=Address2 name2=NameStruct::Test2 newname=Test1 person_age=78 rt=326262362 severity=85"
    );

    let v2 = Top::V2 {
        event_class: "ClassId234",
        name_impl: NameInheritorStruct {
            name_struct: NameStruct {
                name: "Test2".to_owned(),
            },
            name_struct2: Some(NameStruct {
                name: "Test1".to_owned(),
            }),
            address: None,
            age: 78,
        },
        severity: 85,
        unused: 20,
        timestamp: OffsetDateTime::from_unix_timestamp_nanos(9893486324000000),
    };

    assert_eq!(
        v2.to_cef().unwrap(),
        "CEF:1|polyverse|zerotect|V2|ClassId234|Test2|85|EnumV2Field=fixedExtensionsValue EventClassNewName=ClassId234 TopEnumField=fixedExtensionsValue TopStructField=fixedExtensionsValue name2=NameStruct::Test2 newname=Test1 person_age=78 rt=9893486324 severity=85"
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
    fn cef_extensions(
        &self,
        collector: &mut HashMap<String, String>,
    ) -> rust_cef::CefExtensionsResult {
        collector.insert("extension1".to_owned(), "value1".to_owned());
        Ok(())
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
    CefExtensions,
)]
#[cef_values(
    CefHeaderVersion = "1",
    CefHeaderDeviceVendor = "polyverse",
    CefHeaderDeviceProduct = "zerotect"
)]
#[derive(ToCef)]
#[cef_ext_values(TopEnumField = "fixedExtensionsValue")]
enum Top {
    // Name will use the display trait, rather than inheriting the CefHeaderName trait
    #[cef_values(CefHeaderDeviceVersion = "V1")]
    #[cef_ext_values(EnumV1Field = "fixedExtensionsValue")]
    V1(
        #[cef_field(CefHeaderDeviceEventClassID)]
        #[cef_ext_field(top_name)]
        String,
        #[cef_field(CefHeaderName)]
        #[cef_ext_gobble]
        NameInheritorStruct,
        #[cef_field(CefHeaderSeverity)] usize,
        #[cef_ext_gobble] OffsetDateTime,
    ),

    #[cef_values(CefHeaderDeviceVersion = "V2")]
    #[cef_ext_values(EnumV2Field = "fixedExtensionsValue")]
    V2 {
        #[cef_field(CefHeaderDeviceEventClassID)]
        #[cef_ext_field(EventClassNewName)]
        event_class: &'static str,

        #[cef_inherit(CefHeaderName)]
        #[cef_ext_gobble]
        name_impl: NameInheritorStruct,

        #[cef_ext_field]
        #[cef_field(CefHeaderSeverity)]
        severity: usize,

        #[cef_ext_gobble]
        timestamp: OffsetDateTime,

        // `#[allow(dead_code)]` is an attribute that disables the `dead_code` lint
        #[allow(dead_code)]
        unused: usize,
    },
}

#[derive(CefHeaderName)]
struct TupleStule(#[cef_inherit(CefHeaderName)] NameStruct);

#[derive(CefHeaderName, CefExtensions)]
#[cef_ext_values(TopStructField = "fixedExtensionsValue")]
struct NameInheritorStruct {
    // using
    // #[cef_ext_field]
    // would do: name_struct.to_string()
    // but we want to gobble extension field's created inside NameStruct
    #[cef_ext_field(name2)]
    #[cef_inherit(CefHeaderName)]
    pub name_struct: NameStruct,

    #[cef_ext_field]
    pub address: Option<String>,

    #[cef_ext_gobble]
    pub name_struct2: Option<NameStruct>,

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
