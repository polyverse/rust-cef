#[macro_use]
extern crate rust_cef_derive;
use rust_cef::{ToCef};

#[cef_fixed_headers(Version = "0", DeviceVendor="polyverse", DeviceProduct="zerotect", DeviceVersion = "V1", DeviceEventClassID="segfault", Name="attack", Severity="High")]
#[derive(ToCef)]
struct AllFixedHeadersStruct {
}

#[cef_header(Version)]
#[derive(ToCef)]
enum AllDynamicHeadersEnumCEFVersion {
    V0,
    V1,
}

pub enum DeviceVersion {
    V1,
    V2,
}

#[test]
fn test_derive_to_cef() {
    let example1 = AllFixedHeadersStruct {
    };
    let result = example1.to_cef();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "CEF:0|polyverse|zerotect|V1|segfault|attack|High");
}
