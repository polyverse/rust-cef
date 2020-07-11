#[macro_use]
extern crate rust_cef_derive;
use rust_cef::ToCef;

#[derive(ToCef)]
struct Example1 {}

#[derive(ToCef)]
struct Example2 {}

#[derive(ToCef)]
#[cef_fixed_headers()]
struct RealStruct {

}

#[test]
fn test_derive_to_cef() {
    let example1 = Example1 {};
    let _result = example1.to_cef();

    let example2 = Example2 {};
    let _result = example2.to_cef();
}
