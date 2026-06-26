use super::*;

#[test]
fn table_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 4 }
    }));
    assert_table_attr_int(&surface, "viewport_start", 2);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visible_end", 6);
    assert_table_attr_int(&surface, "requested_start", 0);
    assert_table_attr_int(&surface, "requested_count", 8);
    assert_table_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn data_grid_scroll_updates_mui_virtual_window_aliases() {
    let mut surface = table_pointer_route_surface(true, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 3, count: 4 }
    }));
    assert_table_attr_int(&surface, "viewport_start", 3);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visibleEnd", 7);
    assert_table_attr_int(&surface, "requestedStart", 1);
    assert_table_attr_int(&surface, "requestedCount", 8);
    assert_table_attr_float(&surface, "scrollTop", 72.0);
}

#[test]
fn data_grid_disable_virtualization_blocks_default_virtual_scroll() {
    let mut surface =
        table_pointer_route_surface_with_virtualization_options(true, false, false, false, true);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(!result
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SetVisibleRange { .. })));
    assert_table_attr_int(&surface, "viewport_start", 0);
    assert_table_attr_float(&surface, "scrollTop", 0.0);
}
