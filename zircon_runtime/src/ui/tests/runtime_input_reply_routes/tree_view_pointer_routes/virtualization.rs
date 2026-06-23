use super::*;

#[test]
fn tree_view_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    let result = dispatch_tree_scroll(&mut surface, UiPoint::new(24.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }
    }));
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
fn tree_view_virtualized_reparent_drag_updates_window() {
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
