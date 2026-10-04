#[test]
fn hit_query_stacked_outputs_reserve_existing_candidate_bounds() {
    let source = include_str!("../hit_test.rs");
    let implementation = source
        .split("mod output_capacity_tests")
        .next()
        .expect("hit-test implementation before tests");

    assert!(implementation.contains(
        "let mut stacked = Vec::with_capacity(cell.map_or(0, |cell| cell.entries.len()));"
    ));
    assert!(implementation.contains("let mut stacked = Vec::with_capacity(candidates.len());"));
}
