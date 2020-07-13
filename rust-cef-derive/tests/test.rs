/*********************************************************************************************************************/
/* Tests! Tests! Tests! */

#[test]
fn test_derive_without_attributes() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/missing-value-attributes.rs");
}