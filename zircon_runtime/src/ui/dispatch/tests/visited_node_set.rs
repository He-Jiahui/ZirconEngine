use super::*;

#[test]
fn runtime200_typical_ui_route_stays_inline_and_deduplicates() {
    let mut visited = UiDispatchVisitedNodeSet::with_expected_len(10);

    for value in 1..=10 {
        assert!(visited.insert(UiNodeId::new(value)));
    }
    assert!(!visited.insert(UiNodeId::new(4)));
    assert!(!visited.uses_heap_storage());
}

#[test]
fn runtime200_deep_route_promotes_once_and_preserves_membership() {
    let mut visited = UiDispatchVisitedNodeSet::with_expected_len(100);

    for value in 1..=UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY as u64 {
        assert!(visited.insert(UiNodeId::new(value)));
    }
    assert!(!visited.uses_heap_storage());
    assert!(visited.insert(UiNodeId::new(17)));
    assert!(visited.uses_heap_storage());
    assert!(!visited.insert(UiNodeId::new(4)));
    assert!(!visited.insert(UiNodeId::new(17)));
    assert!(visited.insert(UiNodeId::new(18)));
}
