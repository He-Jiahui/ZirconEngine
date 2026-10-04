use super::*;

#[test]
fn empty_membership_is_allocation_free_and_keeps_flat_serde() {
    let entries = UiHitTestCellEntries::default();

    assert!(entries.shared.is_none());
    assert!(entries.is_empty());
    assert_eq!(serde_json::to_string(&entries).unwrap(), "[]");
    assert!(serde_json::from_str::<UiHitTestCellEntries>("[]")
        .unwrap()
        .shared
        .is_none());
}

#[test]
fn retained_membership_clones_only_the_mutated_cell_entries() {
    let retained: UiHitTestCellEntries = vec![1, 2, 3].into();
    let mut next = retained.clone();

    let cloned_entry_count = next.insert(1, 9);

    assert_eq!(cloned_entry_count, 3);
    assert_eq!(retained.as_slice(), &[1, 2, 3]);
    assert_eq!(next.as_slice(), &[1, 9, 2, 3]);
}
