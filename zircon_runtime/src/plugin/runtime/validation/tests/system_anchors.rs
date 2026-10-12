#[test]
fn preallocated_system_anchor_index_preserves_borrowed_registration_contract() {
    let source = include_str!("../system_anchors.rs");
    let capacity_constructor = ["HashSet::with_", "capacity"].concat();
    let capacity_hint = [".size_", "hint()"].concat();
    let unbounded_collect = ["collect::<HashSet", "<_>>()"].concat();

    assert_eq!(source.matches(&capacity_constructor).count(), 1);
    assert_eq!(source.matches(&capacity_hint).count(), 2);
    assert!(!source.contains(&unbounded_collect));
}
