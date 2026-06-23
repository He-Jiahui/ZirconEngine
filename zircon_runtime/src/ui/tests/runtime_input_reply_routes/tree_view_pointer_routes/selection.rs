use super::*;

#[test]
fn tree_view_primary_click_selects_clicked_item_on_owner() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
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
                == UiComponentEvent::SelectOption {
                    property: "value".to_string(),
                    option_id: "Materials".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_string(&surface, "value", "Materials");
    assert_tree_attr_int(&surface, "focused_index", 1);
    assert_tree_attr_int(&surface, "selected_index", 1);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);
}

#[test]
fn tree_view_control_click_toggles_item_in_multi_selection() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_items".to_string(),
                    option_id: "Materials".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Assets", "Materials"]);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );
    let toggled = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );

    assert!(toggled.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_items".to_string(),
                    option_id: "Materials".to_string(),
                    selected: false,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Assets"]);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);
}

#[test]
fn material_tree_view_shift_click_selects_anchor_to_target_range() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        false,
        true,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        false,
        true,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selectedItems".to_string(),
                    option_id: "Shaders".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(
        &surface,
        "selectedItems",
        &["Assets", "Materials", "Shaders"],
    );
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
    assert_tree_attr_int(&surface, "selectionAnchorIndex", 0);
}

#[test]
fn tree_view_double_click_begins_rename_for_clicked_item() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        UiPointerButton::Primary,
        2,
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::KeyboardAction {
                    action: UiComponentKeyboardAction::BeginEdit,
                }
    }));
    assert_tree_attr_bool(&surface, "editing", true);
    assert_tree_attr_string(&surface, "editing_node_id", "Materials");
    assert_tree_attr_string(&surface, "editing_text", "Materials");
    assert_tree_attr_int(&surface, "editing_index", 1);
    assert_tree_attr_bool(&surface, "rename_committed", false);
    assert_tree_attr_string(&surface, "renamed_node_id", "");
    assert_tree_attr_string(&surface, "renamed_text", "");
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_int(&surface, "focused_index", 1);
    assert_tree_attr_int(&surface, "selected_index", 1);
}

#[test]
fn material_tree_view_secondary_release_begins_context_rename_for_clicked_item() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        UiPointerButton::Secondary,
        1,
        false,
        false,
    );
    let result = dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        UiPointerButton::Secondary,
        1,
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::KeyboardAction {
                    action: UiComponentKeyboardAction::BeginEdit,
                }
    }));
    assert_tree_attr_bool(&surface, "editing", true);
    assert_tree_attr_string(&surface, "editingNodeId", "Shaders");
    assert_tree_attr_string(&surface, "editingText", "Shaders");
    assert_tree_attr_int(&surface, "editingIndex", 2);
    assert_tree_attr_bool(&surface, "renameCommitted", false);
    assert_tree_attr_string(&surface, "renamedNodeId", "");
    assert_tree_attr_string(&surface, "renamedText", "");
    assert_tree_attr_strings(&surface, "selectedItems", &["Assets"]);
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
}
