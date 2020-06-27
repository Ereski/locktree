#[test]
fn compiletest() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilefail/*.rs");
    t.pass("tests/compilepass/*.rs");

    #[cfg(feature = "tokio")]
    t.pass("tests/compilepass-tokio/*.rs");
}
