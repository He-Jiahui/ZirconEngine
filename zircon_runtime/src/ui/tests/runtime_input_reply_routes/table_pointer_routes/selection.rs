use super::*;

#[test]
fn table_row_click_selects_row_on_owner() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_rows".to_string(),
                    option_id: "cube".to_string(),
                    selected: true,
                }
    }));
    assert_table_attr_strings(&surface, "selected_rows", &["cube"]);
    assert_table_attr_string(&surface, "value", "cube");
    assert_table_attr_int(&surface, "focused_index", 1);
    assert_table_attr_int(&surface, "selected_index", 1);
}

#[test]
fn table_row_click_projects_configured_typed_row_identity() {
    let mut surface = table_pointer_route_surface(false, false);

    click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));

    assert_table_attr_int(&surface, "selected_row_identity", 73);
}

#[test]
fn table_row_click_resolves_current_typed_identity_into_template_action() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));

    let action = result
        .component_events
        .iter()
        .find_map(|event| event.template_action.as_ref())
        .expect("selected table row should emit its template action");
    assert_eq!(action.route, "test.navigation.surface");
    assert_eq!(
        action.payload.get("surface_entity"),
        Some(&UiValue::Int(73)),
        "template actions must preserve the selected row's typed identity"
    );
}

#[test]
fn selected_row_button_click_uses_current_typed_identity_without_stale_selection() {
    let mut surface = table_pointer_route_surface(false, false);

    click_table_pointer(&mut surface, UiPoint::new(20.0, 52.0));
    let first = click_table_pointer(&mut surface, UiPoint::new(20.0, 135.0));
    let first_action = first
        .component_events
        .iter()
        .find_map(|event| event.template_action.as_ref())
        .expect("selected row button should emit an action");
    assert_eq!(
        first_action.payload.get("surface_entity"),
        Some(&UiValue::Int(41))
    );

    click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));
    let second = click_table_pointer(&mut surface, UiPoint::new(20.0, 135.0));
    let second_action = second
        .component_events
        .iter()
        .find_map(|event| event.template_action.as_ref())
        .expect("updated row selection should emit an action");
    assert_eq!(second_action.route, "test.navigation.bake.surface");
    assert_eq!(
        second_action.payload.get("surface_entity"),
        Some(&UiValue::Int(73))
    );
    assert_eq!(
        second_action.payload.get("force_full_rebuild"),
        Some(&UiValue::Bool(true))
    );
}

#[test]
fn selected_row_button_without_selection_does_not_emit_a_template_action() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 135.0));

    assert!(result
        .component_events
        .iter()
        .all(|event| event.template_action.is_none()));
}

#[test]
fn disabled_selected_row_button_does_not_emit_a_template_action() {
    let mut surface = table_pointer_route_surface(false, false);

    click_table_pointer(&mut surface, UiPoint::new(20.0, 76.0));
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(9))
        .expect("bake-selected button should exist")
        .state_flags
        .enabled = false;

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 135.0));

    assert!(result
        .component_events
        .iter()
        .all(|event| event.template_action.is_none()));
}

#[test]
fn data_grid_row_click_updates_row_selection_model() {
    let mut surface = table_pointer_route_surface(true, false);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 100.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "rowSelectionModel".to_string(),
                    option_id: "camera".to_string(),
                    selected: true,
                }
    }));
    assert_table_attr_strings(&surface, "rowSelectionModel", &["camera"]);
    assert_table_attr_int(&surface, "focused_index", 2);
    assert_table_attr_int(&surface, "selected_index", 2);
}

#[test]
fn data_grid_disable_row_selection_on_click_blocks_row_selection() {
    let mut surface =
        table_pointer_route_surface_with_row_selection_options(true, false, false, true);

    let result = click_table_pointer(&mut surface, UiPoint::new(20.0, 100.0));

    assert!(!result
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SelectOption { .. })));
    assert_table_attr_strings(&surface, "rowSelectionModel", &[]);
    assert_table_attr_missing(&surface, "focused_index");
    assert_table_attr_missing(&surface, "selected_index");
}
