use super::*;

#[test]
fn tree_view_drag_release_reorders_nodes_and_emits_move_element() {
    let mut surface = tree_view_pointer_route_surface(false);

    let begin = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    assert!(begin.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::BeginDrag {
                    property: "nodes".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Begin)
    }));

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "nodes".to_string(),
                    from: 1,
                    to: 2,
                }
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::EndDrag {
                    property: "nodes".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::End)
    }));
    assert_tree_node_order(&surface, "nodes", &["Assets", "Shaders", "Materials"]);
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
}

#[test]
fn material_tree_view_items_reordering_reorders_items_array() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );
    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(24.0, 20.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 20.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "items".to_string(),
                    from: 2,
                    to: 0,
                }
    }));
    assert_tree_node_order(&surface, "items", &["Shaders", "Assets", "Materials"]);
    assert_tree_attr_strings(&surface, "selectedItems", &["Shaders"]);
    assert_tree_attr_int(&surface, "focused_index", 0);
    assert_tree_attr_int(&surface, "selected_index", 0);
}
