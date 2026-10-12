use super::*;

#[test]
fn showcase_action_input_maps_transient_bool_events() {
    assert_eq!(
        demo_input_for_showcase_action("ListRowDemo", "ui_component_showcase.list_row_hovered"),
        UiComponentShowcaseDemoEventInput::Hover(true)
    );
    assert_eq!(
        demo_input_for_showcase_action("ListRowDemo", "ui_component_showcase.list_row_pressed"),
        UiComponentShowcaseDemoEventInput::Press(true)
    );
    assert_eq!(
        demo_input_for_showcase_action(
            "AssetFieldDemo",
            "ui_component_showcase.asset_field_drop_hovered",
        ),
        UiComponentShowcaseDemoEventInput::DropHover(true)
    );
    assert_eq!(
        demo_input_for_showcase_action(
            "AssetFieldDemo",
            "ui_component_showcase.asset_field_active_drag_target",
        ),
        UiComponentShowcaseDemoEventInput::ActiveDragTarget(true)
    );
}

#[test]
fn showcase_action_input_maps_collection_value_changes() {
    assert_eq!(
        demo_input_for_showcase_action(
            "ArrayFieldDemo",
            "ui_component_showcase.array_field_changed",
        ),
        UiComponentShowcaseDemoEventInput::Value(super::collections::demo_array_field_value())
    );
    assert_eq!(
        demo_input_for_showcase_action("MapFieldDemo", "ui_component_showcase.map_field_changed"),
        UiComponentShowcaseDemoEventInput::Value(super::collections::demo_map_field_value())
    );
}

#[test]
fn showcase_action_input_maps_range_field_drag_delta() {
    assert_eq!(
        demo_input_for_showcase_action(
            "RangeFieldDemo",
            "ui_component_showcase.range_field_drag_update",
        ),
        UiComponentShowcaseDemoEventInput::DragDelta(5.0)
    );
}
