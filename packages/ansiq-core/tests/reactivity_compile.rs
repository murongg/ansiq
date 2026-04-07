#[test]
fn reactive_handles_cannot_cross_threads() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/reactive_handles_are_not_send.rs");
}
