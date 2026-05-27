use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentLayoutRole, UiComponentState,
    UiHostCapability, UiRenderCapability, UiValue,
};

use super::super::{assert_has_event, assert_has_prop};

#[test]
fn material_virtualized_descriptors_expose_mui_web_aliases() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let virtual_list = registry
        .descriptor("VirtualList")
        .expect("VirtualList descriptor");
    assert_virtual_range_schema(virtual_list);

    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert_eq!(data_grid.layout_role, UiComponentLayoutRole::VirtualList);
    assert_has_prop(data_grid, "rows");
    assert_has_prop(data_grid, "columns");
    assert_virtual_range_schema(data_grid);
    assert_has_event(data_grid, UiComponentEventKind::SetVisibleRange);
    assert!(data_grid
        .required_host_capabilities
        .contains(&UiHostCapability::VirtualizedLayout));
    assert!(data_grid
        .required_render_capabilities
        .contains(&UiRenderCapability::VirtualizedLayout));
}

#[test]
fn mui_virtual_range_reducer_accepts_react_window_aliases() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let descriptor = registry
        .descriptor("VirtualList")
        .expect("VirtualList descriptor");
    let mut state = UiComponentState::new()
        .with_value("rowCount", UiValue::Int(200))
        .with_value("rowHeight", UiValue::Float(46.0))
        .with_value("overscanCount", UiValue::Int(5));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetVisibleRange {
                start: 10,
                count: 12,
            },
        )
        .unwrap();

    assert_eq!(state.value("total_count"), Some(&UiValue::Int(200)));
    assert_eq!(state.value("row_count"), Some(&UiValue::Int(200)));
    assert_eq!(state.value("rowCount"), Some(&UiValue::Int(200)));
    assert_eq!(state.value("item_count"), Some(&UiValue::Int(200)));
    assert_eq!(state.value("itemCount"), Some(&UiValue::Int(200)));
    assert_eq!(state.value("viewport_start"), Some(&UiValue::Int(10)));
    assert_eq!(state.value("viewport_count"), Some(&UiValue::Int(12)));
    assert_eq!(state.value("visible_end"), Some(&UiValue::Int(22)));
    assert_eq!(state.value("visibleEnd"), Some(&UiValue::Int(22)));
    assert_eq!(state.value("requested_start"), Some(&UiValue::Int(5)));
    assert_eq!(state.value("requestedStart"), Some(&UiValue::Int(5)));
    assert_eq!(state.value("requested_count"), Some(&UiValue::Int(22)));
    assert_eq!(state.value("requestedCount"), Some(&UiValue::Int(22)));
    assert_eq!(state.value("overscan"), Some(&UiValue::Int(5)));
    assert_eq!(state.value("overscanCount"), Some(&UiValue::Int(5)));
    assert_eq!(state.value("scroll_offset"), Some(&UiValue::Float(460.0)));
    assert_eq!(state.value("scrollTop"), Some(&UiValue::Float(460.0)));
}

#[test]
fn mui_data_grid_visible_range_can_derive_count_from_rows_and_disable_virtualization() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let descriptor = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    let mut state = UiComponentState::new()
        .with_value("rows", UiValue::Array(vec![UiValue::Null; 3]))
        .with_value("rowHeight", UiValue::Float(52.0))
        .with_value("disableVirtualization", UiValue::Bool(true));

    state
        .apply_event(
            descriptor,
            UiComponentEvent::SetVisibleRange { start: 1, count: 1 },
        )
        .unwrap();

    assert_eq!(state.value("total_count"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("rowCount"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("viewport_start"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("viewport_count"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("visible_end"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("requested_start"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("requested_count"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("overscan"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("scroll_offset"), Some(&UiValue::Float(0.0)));
}

fn assert_virtual_range_schema(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
) {
    for prop in [
        "total_count",
        "item_count",
        "itemCount",
        "row_count",
        "rowCount",
        "viewport_start",
        "viewport_count",
        "visible_end",
        "requested_start",
        "requested_count",
        "item_extent",
        "itemSize",
        "row_height",
        "rowHeight",
        "overscan",
        "overscan_count",
        "overscanCount",
        "scroll_offset",
        "scrollTop",
        "disable_virtualization",
        "disableVirtualization",
    ] {
        assert_has_prop(descriptor, prop);
    }
}
