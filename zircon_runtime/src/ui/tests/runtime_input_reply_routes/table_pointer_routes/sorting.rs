use super::*;

#[test]
fn table_sort_header_click_toggles_direction_and_sorts_rows() {
    let mut surface = table_pointer_route_surface(false, false);

    let first = click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert!(first.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "sort_column".to_string(),
                    value: UiValue::String("triangles".to_string()),
                }
    }));
    assert_table_sort_state(&surface, "triangles", "asc");
    assert_table_column_sort_indicator(&surface, "triangles", "asc");
    assert_table_row_ids(&surface, ["camera", "cube", "sphere"]);

    click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert_table_sort_state(&surface, "triangles", "desc");
    assert_table_column_sort_indicator(&surface, "triangles", "desc");
    assert_table_row_ids(&surface, ["sphere", "cube", "camera"]);
}

#[test]
fn data_grid_server_sort_header_click_updates_sort_model_without_reordering_rows() {
    let mut surface = table_pointer_route_surface_with_options(true, false, true);

    let result = click_table_pointer(&mut surface, UiPoint::new(190.0, 22.0));

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::ValueChanged {
                    property: "sort_column".to_string(),
                    value: UiValue::String("triangles".to_string()),
                }
    }));
    assert_table_sort_state(&surface, "triangles", "asc");
    assert_table_sort_model(&surface, "triangles", "asc");
    assert_table_row_ids(&surface, ["sphere", "cube", "camera"]);
}
