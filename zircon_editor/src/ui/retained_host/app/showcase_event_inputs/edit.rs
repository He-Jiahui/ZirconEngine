use super::{action_matches, DEFAULT_PAGED_LIST_PAGE_SIZE, DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT};
use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::UiValue;

pub(in crate::ui::retained_host::app) fn demo_input_for_showcase_edit(
    action_id: &str,
    value: &str,
) -> UiComponentShowcaseDemoEventInput {
    if action_matches(action_id, "context_action_menu_open_at") {
        if let Some((x, y)) = parse_popup_anchor(value) {
            return UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y };
        }
    }
    if action_matches(action_id, "virtual_list_scrolled") {
        if let Some(input) = parse_virtual_list_range(value) {
            return input;
        }
    }
    if action_matches(action_id, "paged_list") {
        if let Some(input) = parse_paged_list_request(value) {
            return input;
        }
    }
    if action_matches(action_id, "array_field_remove_element") {
        if let Some(index) = value
            .strip_prefix("array-")
            .and_then(|index| index.parse::<usize>().ok())
        {
            return UiComponentShowcaseDemoEventInput::RemoveElement { index };
        }
    }
    if action_matches(action_id, "array_field_move_element") {
        if let Some((row_id, to)) = value.split_once('=') {
            if let (Some(from), Some(to)) = (
                row_id
                    .strip_prefix("array-")
                    .and_then(|index| index.parse::<usize>().ok()),
                to.parse::<usize>().ok(),
            ) {
                return UiComponentShowcaseDemoEventInput::MoveElement { from, to };
            }
        }
    }
    if action_matches(action_id, "array_field_set_element") {
        if let Some((row_id, value)) = value.split_once('=') {
            if let Some(index) = row_id
                .strip_prefix("array-")
                .and_then(|index| index.parse::<usize>().ok())
            {
                return UiComponentShowcaseDemoEventInput::SetElement {
                    index,
                    value: parse_collection_edit_value(value),
                };
            }
        }
    }
    if action_matches(action_id, "map_field_remove_entry") {
        if let Some(key) = value.strip_prefix("map-") {
            return UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: key.to_string(),
            };
        }
    }
    if action_matches(action_id, "map_field_set_entry") {
        if let Some((row_id, value)) = value.split_once('=') {
            if let Some(key) = row_id.strip_prefix("key:map-") {
                return UiComponentShowcaseDemoEventInput::RenameMapEntry {
                    from_key: key.to_string(),
                    to_key: value.to_string(),
                };
            }
            if let Some(key) = row_id.strip_prefix("map-") {
                return UiComponentShowcaseDemoEventInput::SetMapEntry {
                    key: key.to_string(),
                    value: parse_collection_edit_value(value),
                };
            }
        }
    }
    let value =
        if action_matches(action_id, "number_field") || action_matches(action_id, "range_field") {
            value
                .parse::<f64>()
                .map(UiValue::Float)
                .unwrap_or_else(|_| UiValue::String(value.to_string()))
        } else {
            UiValue::String(value.to_string())
        };
    UiComponentShowcaseDemoEventInput::Value(value)
}

fn parse_collection_edit_value(value: &str) -> UiValue {
    if let Ok(value) = value.parse::<bool>() {
        return UiValue::Bool(value);
    }
    value
        .parse::<f64>()
        .map(UiValue::Float)
        .unwrap_or_else(|_| UiValue::String(value.to_string()))
}

fn parse_popup_anchor(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

fn parse_virtual_list_range(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (start, count) = parse_i64_request_pair(
        value,
        &["start", "viewport_start", "requested_start"],
        &["count", "viewport_count", "requested_count"],
        DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
    )?;
    Some(UiComponentShowcaseDemoEventInput::SetVisibleRange { start, count })
}

fn parse_paged_list_request(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (page_index, page_size) = parse_i64_request_pair(
        value,
        &["page", "page_index", "index"],
        &["size", "page_size"],
        DEFAULT_PAGED_LIST_PAGE_SIZE,
    )?;
    Some(UiComponentShowcaseDemoEventInput::SetPage {
        page_index,
        page_size,
    })
}

fn parse_i64_request_pair(
    value: &str,
    first_keys: &[&str],
    second_keys: &[&str],
    default_second: i64,
) -> Option<(i64, i64)> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.contains('=') {
        let mut first = None;
        let mut second = None;
        for part in value.split([',', ';', '&']) {
            let (key, raw_value) = part.split_once('=')?;
            let key = key.trim();
            let parsed_value = raw_value.trim().parse::<i64>().ok()?;
            if first_keys.iter().any(|candidate| key == *candidate) {
                first = Some(parsed_value);
            } else if second_keys.iter().any(|candidate| key == *candidate) {
                second = Some(parsed_value);
            }
        }
        return first.map(|first| (first, second.unwrap_or(default_second)));
    }
    if let Some((first, second)) = value.split_once(',') {
        return Some((first.trim().parse().ok()?, second.trim().parse().ok()?));
    }
    value
        .parse::<i64>()
        .ok()
        .map(|first| (first, default_second))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            demo_input_for_showcase_edit(
                "ui_component_showcase.map_field_set_entry",
                "map-speed=2.5"
            ),
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
            demo_input_for_showcase_edit(
                "ui_component_showcase.input_field_committed",
                "committed"
            ),
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
}
