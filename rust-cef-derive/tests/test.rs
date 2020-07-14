#[macro_use]
extern crate rust_cef_derive;

use rust_cef::{CefHeaderVersion, CefHeaderName};

/*********************************************************************************************************************/
/* Tests! Tests! Tests! */

#[cef_fixed_headers(Version = "3235", Name = "name2")]
struct MultipleHeaders {}

#[cef_fixed_headers(Version = "3424")]
#[cef_fixed_headers(Name = "name1")]
struct MultipleAttrs {}

#[cef_fixed_headers(Version = "4234")]
struct SingleHeader {}

#[cef_fixed_headers(Version = "0", DeviceVendor = "polyverse", DeviceProduct = "zerotect", DeviceVersion = "V1", DeviceEventClassID = "LinuxKernelFault", Name = "Linux Kernel Fault", Severity = "10")]
#[derive(ToCef)]
struct AllFixedHeaders {}

#[test]
fn test_cef_fixed_headers_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cef-fixed-headers-negative.rs");
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
