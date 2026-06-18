use std::collections::BTreeMap;

use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::{UiDragPayload, UiDragPayloadKind, UiValue};

use super::{
    action_matches, action_matches_binding_suffix, select_option, DEFAULT_PAGED_LIST_PAGE_SIZE,
    DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
};

const ASSET_FIELD_CLEAR_BINDING_SUFFIX: &str = "AssetFieldClear";
const ASSET_FIELD_LOCATE_BINDING_SUFFIX: &str = "AssetFieldLocate";
const ASSET_FIELD_OPEN_BINDING_SUFFIX: &str = "AssetFieldOpen";

pub(in crate::ui::retained_host::app) fn demo_input_for_showcase_action(
    control_id: &str,
    action_id: &str,
) -> UiComponentShowcaseDemoEventInput {
    match action_id {
        action if action_matches(action, "number_field_drag_update") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action_matches(action, "number_field_large_drag_update") => {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(1.0)
        }
        action if action_matches(action, "number_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0))
        }
        action if action_matches(action, "range_field_drag_update") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action_matches(action, "range_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(72.0))
        }
        action if action_matches(action, "color_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Color("#ffcc33".to_string()))
        }
        action if action_matches(action, "vector2_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec2([16.0, 32.0]))
        }
        action if action_matches(action, "vector3_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec3([3.0, 4.0, 5.0]))
        }
        action if action_matches(action, "vector4_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec4([0.25, 0.5, 0.75, 1.0]))
        }
        action if action_matches(action, "input_field") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::String(
                "Runtime UI event".to_string(),
            ))
        }
        action if action_matches(action, "text_field") => UiComponentShowcaseDemoEventInput::Value(
            UiValue::String("Runtime UI event-driven text".to_string()),
        ),
        action
            if action_matches(action, "toggle_button_changed")
                || action_matches(action, "checkbox_changed") =>
        {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action_matches(action, "radio_changed") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action_matches(action, "segmented_control_changed") => {
            select_option("rotate", true)
        }
        action if action_matches(action, "tab_changed") => select_option("scene", true),
        action if action_matches(action, "tab_strip_changed") => select_option("assets", true),
        action if action_matches(action, "slider_drag_update") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action_matches(action, "slider_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0))
        }
        action if action_matches(action, "range_slider_drag_update") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action_matches(action, "range_slider_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(78.0))
        }
        action if action_matches(action, "dropdown_changed") => select_option("editor", true),
        action if action_matches(action, "combo_box_changed") => select_option("native", true),
        action if action_matches(action, "enum_field_changed") => {
            select_option("UnityInspector", true)
        }
        action if action_matches(action, "flags_field_changed") => select_option("Disabled", true),
        action if action_matches(action, "search_select_changed") => {
            select_option("runtime.ui.RangeField", true)
        }
        action if action_matches(action, "search_select_query_changed") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("vector".to_string()))
        }
        action if action_matches(action, "context_action_menu_open_at") => {
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 184.0, y: 88.0 }
        }
        action if action_matches(action, "context_action_menu_changed") => {
            select_option("Open Source", true)
        }
        action if action_matches(action, "asset_field_dropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://materials/runtime_demo.mat",
                ),
            }
        }
        action if action_matches(action, "asset_field_drop_hovered") => {
            UiComponentShowcaseDemoEventInput::DropHover(true)
        }
        action if action_matches(action, "asset_field_active_drag_target") => {
            UiComponentShowcaseDemoEventInput::ActiveDragTarget(true)
        }
        action
            if action_matches_binding_suffix(action, ASSET_FIELD_CLEAR_BINDING_SUFFIX)
                || action_matches_binding_suffix(action, ASSET_FIELD_LOCATE_BINDING_SUFFIX)
                || action_matches_binding_suffix(action, ASSET_FIELD_OPEN_BINDING_SUFFIX) =>
        {
            UiComponentShowcaseDemoEventInput::None
        }
        action if action_matches(action, "instance_field_dropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::SceneInstance,
                    "scene://Root/RuntimeDemoLight",
                ),
            }
        }
        action if action_matches(action, "object_field_dropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Object,
                    "object://Selection/RuntimeDemo",
                ),
            }
        }
        action if action_matches(action, "group_toggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action_matches(action, "foldout_toggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action_matches(action, "inspector_section_toggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action_matches(action, "tree_row_toggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action_matches(action, "array_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(demo_array_field_value())
        }
        action if action_matches(action, "array_field_add_element") => {
            UiComponentShowcaseDemoEventInput::AddElement {
                value: UiValue::String("MapField".to_string()),
            }
        }
        action if action_matches(action, "array_field_set_element") => {
            UiComponentShowcaseDemoEventInput::SetElement {
                index: 1,
                value: UiValue::String("Vector3Field".to_string()),
            }
        }
        action if action_matches(action, "array_field_remove_element") => {
            UiComponentShowcaseDemoEventInput::RemoveElement { index: 0 }
        }
        action if action_matches(action, "array_field_move_element") => {
            UiComponentShowcaseDemoEventInput::MoveElement { from: 0, to: 1 }
        }
        action if action_matches(action, "map_field_changed") => {
            UiComponentShowcaseDemoEventInput::Value(demo_map_field_value())
        }
        action if action_matches(action, "map_field_add_entry") => {
            UiComponentShowcaseDemoEventInput::AddMapEntry {
                key: "layer".to_string(),
                value: UiValue::String("Editor".to_string()),
            }
        }
        action if action_matches(action, "map_field_set_entry") => {
            UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            }
        }
        action if action_matches(action, "map_field_remove_entry") => {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: "speed".to_string(),
            }
        }
        action if action_matches(action, "list_row_hovered") => {
            UiComponentShowcaseDemoEventInput::Hover(true)
        }
        action if action_matches(action, "list_row_pressed") => {
            UiComponentShowcaseDemoEventInput::Press(true)
        }
        action if action_matches(action, "list_row_clicked") => {
            UiComponentShowcaseDemoEventInput::None
        }
        action if action_matches(action, "virtual_list_scrolled") => {
            UiComponentShowcaseDemoEventInput::SetVisibleRange {
                start: 240,
                count: DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
            }
        }
        action if action_matches(action, "paged_list_next_page") => {
            UiComponentShowcaseDemoEventInput::SetPage {
                page_index: 1,
                page_size: DEFAULT_PAGED_LIST_PAGE_SIZE,
            }
        }
        action if action_matches(action, "world_space_surface_moved") => {
            UiComponentShowcaseDemoEventInput::SetWorldTransform {
                position: [1.0, 2.0, 4.0],
                rotation: [0.0, 180.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            }
        }
        action if action_matches(action, "world_space_surface_configured") => {
            UiComponentShowcaseDemoEventInput::SetWorldSurface {
                size: [2.5, 1.25],
                pixels_per_meter: 256.0,
                billboard: true,
                depth_test: true,
                render_order: 4,
                camera_target: "viewport-main".to_string(),
            }
        }
        action if action_matches(action, "show") && control_id.starts_with("ComponentShowcase") => {
            UiComponentShowcaseDemoEventInput::None
        }
        _ => UiComponentShowcaseDemoEventInput::None,
    }
}

fn demo_array_field_value() -> UiValue {
    UiValue::Array(vec![
        UiValue::String("Label".to_string()),
        UiValue::String("Transform".to_string()),
        UiValue::String("Material".to_string()),
    ])
}

fn demo_map_field_value() -> UiValue {
    let mut entries = BTreeMap::new();
    entries.insert("speed".to_string(), UiValue::Float(2.5));
    entries.insert("visible".to_string(), UiValue::Bool(false));
    UiValue::Map(entries)
}

#[cfg(test)]
mod tests {
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
            UiComponentShowcaseDemoEventInput::Value(demo_array_field_value())
        );
        assert_eq!(
            demo_input_for_showcase_action(
                "MapFieldDemo",
                "ui_component_showcase.map_field_changed"
            ),
            UiComponentShowcaseDemoEventInput::Value(demo_map_field_value())
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
}
