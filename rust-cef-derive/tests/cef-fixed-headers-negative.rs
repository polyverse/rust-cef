#[macro_use]
extern crate rust_cef_derive;

#[cef_fixed_headers()]
struct MissingAttributeValues {}

#[cef_fixed_headers]
struct MissingAttributeList {}

#[cef_fixed_headers(Version = "0", SomethingElse = "1", Version = "2")]
struct InvalidHeader {}

#[cef_fixed_headers(Version)]
struct SingleIdent {}

#[cef_fixed_headers(Version + 10)]
struct Expr {}

#[cef_fixed_headers(Version, Name)]
struct MultipleIdents {}

#[cef_fixed_headers(Version = 0)]
struct NonStringHeader {}

#[cef_fixed_headers(Version = "0")]
union ApplyToUnion {}

#[cef_fixed_headers(Version = "0")]
fn ApplyToFn() -> () {}

#[cef_fixed_headers(Version = "0")]
const APPLY_TO_CONST: &str = "";

struct ApplyToField {
    #[cef_fixed_headers(Version = "0")]
    pub field: usize
}

#[cef_fixed_headers(Version = "0", Version = "2")]
struct DuplicateHeaders1{}

#[cef_fixed_headers(Version = "0")]
#[cef_fixed_headers(Version = "0")]
struct DuplicateHeaders2 {}

fn main() {

}