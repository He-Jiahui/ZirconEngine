pub(in crate::tests::runtime_absorption::plan_status) fn assert_contains_all(
    label: &str,
    source: &str,
    anchors: &[&str],
) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should keep runtime plan-status anchor `{anchor}`"
        );
    }
}
