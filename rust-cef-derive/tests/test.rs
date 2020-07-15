#[macro_use]
extern crate rust_cef_derive;

use rust_cef::{ToCef, CefHeaderVersion, CefHeaderName, CefExtensions, CefHeaderSeverity};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[test]
fn test_cef_fixed_headers_fails() {
    let t = trybuild::TestCases::new();
    //t.compile_fail("tests/cef-fixed-headers-negative.rs");
    //t.compile_fail("tests/cef-inherit-negative.rs");
}


#[test]
fn test_cef_fixed_headers() {
    let sh = SingleHeader{};
    assert_eq!(sh.cef_header_version().unwrap(), "4234");

    let ma = MultipleAttrs{};
    assert_eq!(ma.cef_header_version().unwrap(), "3424");
    assert_eq!(ma.cef_header_name().unwrap(), "name1");

    let mh = MultipleHeaders {};
    assert_eq!(mh.cef_header_version().unwrap(), "3235");
    assert_eq!(mh.cef_header_name().unwrap(), "name2");
}

#[test]
fn test_to_cef_with_fixed_headers_and_custom_extensions() {
    let t = AllFixedHeadersCustomExtensions{};
    assert_eq!(t.to_cef().unwrap(), "CEF:0|polyverse|zerotect|V1|LinuxKernelFault|Linux Kernel Fault|10|extension1=value1")
}


#[test]
fn test_to_cef_with_fixed_and_manual_headers() {
    let t = ManualAndFixedHeaders{};
    assert_eq!(t.to_cef().unwrap(), "CEF:customVersion|polyverse|zerotect|V1|LinuxKernelFault|Linux Kernel Fault|10|")
}

#[test]
fn test_cef_inherit() {
    let t1 = Top::V1("ClassId234".to_owned(), NameInheritorStruct{name_struct: NameStruct{name: "Test1".to_owned()}}, 24);
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

#[derive(CefHeaderVersion, CefHeaderDeviceVendor, CefHeaderDeviceProduct, CefHeaderDeviceVersion, CefHeaderDeviceEventClassID, CefHeaderName, CefHeaderSeverity)]
#[cef_values(CefHeaderVersion = "0", CefHeaderDeviceVendor = "polyverse", CefHeaderDeviceProduct = "zerotect", CefHeaderDeviceVersion = "V1", CefHeaderDeviceEventClassID = "LinuxKernelFault", CefHeaderName = "Linux Kernel Fault", CefHeaderSeverity = "10")]
#[derive(ToCef)]
struct AllFixedHeadersCustomExtensions {}
impl CefExtensions for AllFixedHeadersCustomExtensions {
    fn cef_extensions(&self) -> rust_cef::CefResult {
        Ok("extension1=value1".to_owned())
    }
}

#[derive(CefHeaderDeviceVendor, CefHeaderDeviceProduct, CefHeaderDeviceVersion, CefHeaderDeviceEventClassID)]
#[derive(CefHeaderName, CefHeaderSeverity)]
#[cef_values(CefHeaderDeviceVendor = "polyverse", CefHeaderDeviceProduct = "zerotect")]
#[cef_values(CefHeaderName = "Linux Kernel Fault", CefHeaderSeverity = "10", CefHeaderDeviceVersion = "V1", CefHeaderDeviceEventClassID = "LinuxKernelFault")]
#[derive(ToCef, CefExtensions)]
struct ManualAndFixedHeaders {}
impl CefHeaderVersion for ManualAndFixedHeaders {
    fn cef_header_version(&self) -> rust_cef::CefResult {
        Ok("customVersion".to_owned())
    }
}


#[derive(CefHeaderVersion, CefHeaderDeviceVendor, CefHeaderDeviceVersion, CefHeaderDeviceEventClassID)]
#[derive(CefHeaderName, CefHeaderDeviceProduct, CefHeaderSeverity)]
#[cef_values(CefHeaderVersion = "0", CefHeaderDeviceVendor = "polyverse", CefHeaderDeviceProduct = "zerotect")]
#[derive(ToCef, CefExtensions)]
enum Top {

    // Name will use the display trait, rather than inheriting the CefHeaderName trait
    #[cef_values(CefHeaderDeviceVersion = "V1", CefHeaderName = 1, CefHeaderSeverity = 2, CefHeaderDeviceEventClassID = 0)]
    V1(String, NameInheritorStruct, usize),

    #[cef_values(CefHeaderDeviceVersion = "V2", CefHeaderSeverity = severity)]
    #[cef_inherit(Name = name_impl)]
    V2{event_class: &'static str, name_impl: NameInheritorStruct, severity: usize}
}

#[derive(CefHeaderName)]
struct NameInheritorStruct {

    #[cef_inherit(CefHeaderName)]
    pub name_struct: NameStruct
}
impl Display for NameInheritorStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NameInheritorStruct::{}", self.name_struct)
    }
}


#[derive(CefHeaderName)]
struct NameStruct {

    #[cef_values(CefHeaderName)]
    pub name: String
}
impl Display for NameStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "NameStruct::{}", self.name)
    }
}
