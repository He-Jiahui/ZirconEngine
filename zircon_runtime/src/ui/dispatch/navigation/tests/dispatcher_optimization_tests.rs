#[test]
fn navigation_dispatch_reserves_candidate_invocation_upper_bound() {
    let source = include_str!("../dispatcher.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("navigation dispatcher implementation before tests");

    assert!(implementation.contains("let mut invocations = Vec::with_capacity(candidates.len());"));
}
