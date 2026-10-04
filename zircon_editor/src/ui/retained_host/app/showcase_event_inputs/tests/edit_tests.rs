use super::super::{DEFAULT_PAGED_LIST_PAGE_SIZE, DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT};
use super::*;
use zircon_runtime_interface::ui::component::UiValue;

#[test]
fn showcase_edit_input_maps_collection_row_payloads_to_typed_events() {
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.array_field_set_element",
            "array-0=Vector3Field",
        ),
        UiComponentShowcaseDemoEventInput::SetElement {
            index: 0,
            value: UiValue::String("Vector3Field".to_string()),
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.map_field_set_entry", "map-speed=2.5"),
        UiComponentShowcaseDemoEventInput::SetMapEntry {
            key: "speed".to_string(),
            value: UiValue::Float(2.5),
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.map_field_set_entry",
            "map-visible=false"
        ),
        UiComponentShowcaseDemoEventInput::SetMapEntry {
            key: "visible".to_string(),
            value: UiValue::Bool(false),
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.map_field_set_entry",
            "key:map-speed=velocity",
        ),
        UiComponentShowcaseDemoEventInput::RenameMapEntry {
            from_key: "speed".to_string(),
            to_key: "velocity".to_string(),
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.array_field_move_element",
            "array-2=1",
        ),
        UiComponentShowcaseDemoEventInput::MoveElement { from: 2, to: 1 }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.array_field_remove_element",
            "array-1",
        ),
        UiComponentShowcaseDemoEventInput::RemoveElement { index: 1 }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.map_field_remove_entry",
            "map-visible",
        ),
        UiComponentShowcaseDemoEventInput::RemoveMapEntry {
            key: "visible".to_string(),
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.context_action_menu_open_at",
            "212,96",
        ),
        UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 212.0, y: 96.0 }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.input_field_committed", "committed"),
        UiComponentShowcaseDemoEventInput::Value(UiValue::String("committed".to_string()))
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.number_field_committed", "51"),
        UiComponentShowcaseDemoEventInput::Value(UiValue::Float(51.0))
    );
}

#[test]
fn showcase_edit_input_maps_virtual_list_scroll_payload_to_visible_range() {
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.virtual_list_scrolled",
            "start=512,count=48",
        ),
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start: 512,
            count: 48,
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.virtual_list_scrolled", "128,24"),
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start: 128,
            count: 24,
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.virtual_list_scrolled", "240"),
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start: 240,
            count: DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
        }
    );
}

#[test]
fn showcase_edit_input_maps_paged_list_payload_to_page_request() {
    assert_eq!(
        demo_input_for_showcase_edit(
            "ui_component_showcase.paged_list_next_page",
            "page=3,size=100",
        ),
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: 3,
            page_size: 100,
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.paged_list_go_to_page", "4,50"),
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: 4,
            page_size: 50,
        }
    );
    assert_eq!(
        demo_input_for_showcase_edit("ui_component_showcase.paged_list_previous_page", "2"),
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: 2,
            page_size: DEFAULT_PAGED_LIST_PAGE_SIZE,
        }
    );
}
