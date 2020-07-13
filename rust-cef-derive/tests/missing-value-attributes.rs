#[macro_use]
extern crate rust_cef_derive;

#[derive(
    CefHeaderVersion,
    CefHeaderDeviceVendor,
    CefHeaderDeviceProduct,
    CefHeaderDeviceVersion,
    CefHeaderDeviceEventClassID,
    CefHeaderName,
    CefHeaderSeverity,
    CefExtensions,

)]
struct MissingValueAttributes {}

fn main() {

}