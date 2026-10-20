//! 虚拟树滚动与跨节点拖拽共同维护可见窗口；到边界不发布空范围事务。

use super::*;

#[test]
fn tree_view_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    let result = dispatch_tree_scroll(&mut surface, UiPoint::new(24.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }
    }));
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].applied_count, 32);
    assert_eq!(result.binding_reports[0].updates.len(), 32);
    assert_tree_attr_int(&surface, "total_count", 7);
    assert_tree_attr_int(&surface, "item_count", 7);
    assert_tree_attr_int(&surface, "viewport_start", 2);
    assert_tree_attr_int(&surface, "viewport_count", 3);
    assert_tree_attr_int(&surface, "visible_end", 5);
    assert_tree_attr_int(&surface, "visibleEnd", 5);
    assert_tree_attr_int(&surface, "requested_start", 1);
    assert_tree_attr_int(&surface, "requested_count", 5);
    assert_tree_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn tree_view_virtual_window_reparent_drag_updates_window() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 92.0),
        false,
        false,
    );
    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "nodes".to_string(),
                    from: 3,
                    to: 2,
                }
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }
    }));
    assert_tree_node_order(&surface, "nodes", &["Assets", "Scenes"]);
    assert_tree_child_order(&surface, "nodes", "Materials", &["Shaders"]);
    assert_tree_child_order(&surface, "nodes", "Shaders", &["Vertex", "Fragment"]);
    assert_tree_attr_strings(&surface, "selected_items", &["Shaders"]);
    assert_tree_attr_strings(
        &surface,
        "expanded_items",
        &["Assets", "Shaders", "Materials"],
    );
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
    assert_tree_attr_int(&surface, "selection_anchor_index", 2);
    assert_tree_attr_int(&surface, "viewport_start", 2);
    assert_tree_attr_int(&surface, "visible_end", 5);
    assert_tree_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn tree_view_virtual_window_at_boundary_emits_no_transaction() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    let first = dispatch_tree_scroll(&mut surface, UiPoint::new(24.0, 76.0), 10_000.0);
    let boundary = dispatch_tree_scroll(&mut surface, UiPoint::new(24.0, 76.0), 10_000.0);

    assert_eq!(first.binding_reports.len(), 1);
    assert!(boundary.binding_reports.is_empty());
    assert!(!boundary
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SetVisibleRange { .. })));
    assert_tree_attr_int(&surface, "viewport_start", 4);
}

#[test]
fn tree_view_overscan_near_first_row_keeps_scroll_request_within_real_rows() {
    for material in [false, true] {
        let mut surface = tree_view_virtualized_reparent_surface(material);
        surface
            .tree
            .node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("overscan".to_string(), toml::Value::Integer(2));
        let point = UiPoint::new(24.0, 76.0);
        let forward = dispatch_tree_scroll(&mut surface, point, 24.0);
        assert_eq!(forward.binding_reports.len(), 1);
        assert!(forward
            .component_events
            .iter()
            .any(|event| event.target == UiNodeId::new(2)
                && event.event == UiComponentEvent::SetVisibleRange { start: 1, count: 3 }));
        assert_tree_attr_int(&surface, "viewport_start", 1);
        assert_tree_attr_int(&surface, "requested_start", 0);
        assert_tree_attr_int(&surface, "requested_count", 6);
        assert_tree_attr_int(&surface, "requestedStart", 0);
        assert_tree_attr_int(&surface, "requestedCount", 6);
        let back = dispatch_tree_scroll(&mut surface, point, -24.0);
        assert_eq!(back.binding_reports.len(), 1);
        assert_tree_attr_int(&surface, "viewport_start", 0);
        assert_tree_attr_int(&surface, "requested_start", 0);
        assert_tree_attr_int(&surface, "requested_count", 5);
    }
}

#[test]
fn tree_view_reparent_clamps_overscan_before_publishing_requested_count() {
    for material in [false, true] {
        let mut surface = tree_view_virtualized_reparent_surface(material);
        surface
            .tree
            .node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("overscan".to_string(), toml::Value::Integer(3));
        dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Down,
            UiPoint::new(24.0, 92.0),
            false,
            false,
        );
        dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Move,
            UiPoint::new(24.0, 44.0),
            false,
            false,
        );
        let result = dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Up,
            UiPoint::new(24.0, 44.0),
            false,
            false,
        );
        assert!(result
            .component_events
            .iter()
            .any(|event| event.target == UiNodeId::new(2)
                && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }));
        assert_tree_attr_int(&surface, "viewport_start", 2);
        assert_tree_attr_int(&surface, "requested_start", 0);
        assert_tree_attr_int(&surface, "requested_count", 7);
        assert_tree_attr_int(&surface, "requestedStart", 0);
        assert_tree_attr_int(&surface, "requestedCount", 7);
        assert_tree_child_order(
            &surface,
            if material { "items" } else { "nodes" },
            "Materials",
            &["Shaders"],
        );
    }
}
