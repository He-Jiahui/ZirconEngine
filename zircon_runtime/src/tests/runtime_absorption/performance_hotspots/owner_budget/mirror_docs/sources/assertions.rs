pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should mirror Runtime 07 performance-hotpath audit anchor `{anchor}`"
        );
    }
}
