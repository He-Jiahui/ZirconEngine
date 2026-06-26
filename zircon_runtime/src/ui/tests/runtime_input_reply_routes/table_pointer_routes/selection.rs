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
