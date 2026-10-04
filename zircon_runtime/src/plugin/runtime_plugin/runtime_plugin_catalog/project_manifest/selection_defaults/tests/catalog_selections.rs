#[test]
fn preallocated_catalog_selection_completion_preserves_behavior_contract() {
    let source = include_str!("../catalog_selections.rs");
    let preallocated_set = ["HashSet::with_", "capacity"].concat();
    let target_reserve = ["completed.selections.", "reserve(registrations.len())"].concat();
    let unbounded_collect = ["collect::<HashSet", "<_>>()"].concat();

    assert_eq!(source.matches(&preallocated_set).count(), 1);
    assert_eq!(source.matches(&target_reserve).count(), 1);
    assert!(!source.contains(&unbounded_collect));
}
