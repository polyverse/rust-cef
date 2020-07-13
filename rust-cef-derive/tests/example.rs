#[macro_use]
extern crate rust_cef_derive;
use rust_cef::ToCef;

#[derive(
    ToCef,
    CefHeaderVersion,
    CefHeaderDeviceVendor,
    CefHeaderDeviceProduct,
    CefHeaderDeviceVersion,
    CefHeaderDeviceEventClassID,
    CefHeaderName,
    CefHeaderSeverity,
    CefExtensions,
)]
struct AllFixedHeadersStruct {}

#[test]
fn test_derive_to_cef() {
    let example1 = AllFixedHeadersStruct {};
    let result = example1.to_cef();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "CEF:0|polyverse|zerotect|V1|segfault|attack|High"
    );
}
