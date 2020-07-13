#[macro_use]
extern crate rust_cef_derive;

#[cef_fixed_headers(Version = "0", Name = "foo")]
struct MissingValueAttributes {}

fn main() {

}