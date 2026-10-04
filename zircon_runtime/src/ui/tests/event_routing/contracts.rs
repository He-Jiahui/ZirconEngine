//! 路由复用与派发热路径的现有源码合同，由实际所有者文件提供检查输入。

#[test]
fn pointer_dispatch_does_not_clone_an_unused_hover_path() {
    let source = include_str!("../../surface/surface/event_routing.rs");

    assert!(
        !source.contains("_hover_before_dispatch"),
        "pointer dispatch must not clone the hovered path before routing when that copy is unused"
    );
}

#[test]
fn pointer_hot_path_reuses_hit_routes_and_skips_empty_handler_contexts() {
    let routing_source = include_str!("../../surface/surface/event_routing/pointer_ownership.rs");
    let dispatcher_source = include_str!("../../dispatch/pointer/dispatcher.rs");

    assert!(
        routing_source.contains("UiPointerRoutingPath::HitPath"),
        "top-hit pointer routing should reuse the canonical hit path"
    );
    assert!(
        !routing_source.contains("hit.path.bubble_route.clone()"),
        "top-hit pointer routing must not copy the hit path"
    );
    assert!(
        dispatcher_source.contains("if self.handlers.is_empty() && self.phase_handlers.is_empty()"),
        "the default pointer dispatcher should bypass route traversal entirely"
    );
    assert!(
        dispatcher_source.contains("if phase_handlers.is_none() && unqualified_handlers.is_none()"),
        "nodes without pointer handlers must not allocate a dispatch context"
    );
}

#[test]
fn default_table_sort_borrows_common_scalar_text() {
    let source = include_str!("../../surface/surface/default_interactions/table/columns.rs");

    assert!(
        source.contains("left.and_then(borrowed_sort_text)"),
        "default table sorting should compare common string-like values without allocating display text"
    );
}
