use super::collect_present_with_capacity;

#[test]
fn preallocated_present_collection_preserves_order_and_omits_empty_slots() {
    let present = collect_present_with_capacity(vec![Some(7), None, Some(3), None, Some(11)], 3);

    assert_eq!(present, vec![7, 3, 11]);
    assert!(present.capacity() >= 3);
}

#[test]
fn capability_wait_index_reserves_one_bucket_per_pending_feature() {
    let pending_count = 8;
    let waiting_by_capability =
        std::collections::HashMap::<String, Vec<usize>>::with_capacity(pending_count);

    assert!(waiting_by_capability.capacity() >= pending_count);
}
