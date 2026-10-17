//! 验证浅路由不升格，以及升格前后重复节点仍保持一致判定。

use super::*;

// 浅路由重复项应被拒绝，且不能仅因插入而分配堆集合。
#[test]
fn runtime200_typical_ui_route_stays_inline_and_deduplicates() {
    let mut visited = UiDispatchVisitedNodeSet::with_expected_len(10);

    for value in 1..=10 {
        assert!(visited.insert(UiNodeId::new(value)));
    }
    assert!(!visited.insert(UiNodeId::new(4)));
    assert!(!visited.uses_heap_storage());
}

// 容量边界后的升格需保留旧成员，再对新旧节点统一判重。
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
