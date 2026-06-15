use std::collections::BTreeMap;

use toml::Value;

#[derive(Clone, Debug, Default)]
pub(super) struct ProjectedDragOverlayData {
    pub popup_open: Option<bool>,
    pub text: Option<String>,
    pub value_text: Option<String>,
    pub payload_kind: String,
    pub payload_label: String,
    pub payload_reference: String,
    pub has_cursor: bool,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub preview_width: f32,
    pub preview_height: f32,
    pub drop_allowed: bool,
    pub has_drop_target: bool,
    pub drop_target_x: f32,
    pub drop_target_y: f32,
    pub drop_target_width: f32,
    pub drop_target_height: f32,
    pub drop_indicator_edge: String,
    pub drop_indicator_text: String,
}

pub(super) fn projected_drag_overlay_data(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> ProjectedDragOverlayData {
    if component_role != "drag-overlay" {
        return ProjectedDragOverlayData::default();
    }

    let open = bool_attribute(attributes, "open").unwrap_or(false)
        || bool_attribute(attributes, "dragging").unwrap_or(false);
    let cursor_x = f32_attribute(attributes, "cursor_x");
    let cursor_y = f32_attribute(attributes, "cursor_y");
    let drop_target_x = f32_attribute(attributes, "drop_target_x");
    let drop_target_y = f32_attribute(attributes, "drop_target_y");
    let drop_target_width = f32_attribute(attributes, "drop_target_width");
    let drop_target_height = f32_attribute(attributes, "drop_target_height");
    let payload_label = first_non_empty_string_attribute(
        attributes,
        &["payload_label", "label", "text", "payload_reference"],
    )
    .unwrap_or_default();
    let payload_reference = string_attribute(attributes, "payload_reference").unwrap_or_default();

    ProjectedDragOverlayData {
        popup_open: Some(open),
        text: (!payload_label.is_empty()).then(|| payload_label.clone()),
        value_text: (!payload_reference.is_empty()).then(|| payload_reference.clone()),
        payload_kind: string_attribute(attributes, "payload_kind")
            .unwrap_or_else(|| "unknown".to_string()),
        payload_label,
        payload_reference,
        has_cursor: cursor_x.is_some() && cursor_y.is_some(),
        cursor_x: cursor_x.unwrap_or(0.0),
        cursor_y: cursor_y.unwrap_or(0.0),
        offset_x: f32_attribute(attributes, "offset_x").unwrap_or(12.0),
        offset_y: f32_attribute(attributes, "offset_y").unwrap_or(12.0),
        preview_width: f32_attribute(attributes, "preview_width").unwrap_or(0.0),
        preview_height: f32_attribute(attributes, "preview_height").unwrap_or(0.0),
        drop_allowed: bool_attribute(attributes, "drop_allowed").unwrap_or(true),
        has_drop_target: drop_target_x.is_some()
            && drop_target_y.is_some()
            && drop_target_width.is_some()
            && drop_target_height.is_some(),
        drop_target_x: drop_target_x.unwrap_or(0.0),
        drop_target_y: drop_target_y.unwrap_or(0.0),
        drop_target_width: drop_target_width.unwrap_or(0.0),
        drop_target_height: drop_target_height.unwrap_or(0.0),
        drop_indicator_edge: string_attribute(attributes, "drop_indicator_edge")
            .unwrap_or_else(|| "none".to_string()),
        drop_indicator_text: string_attribute(attributes, "drop_indicator_text")
            .unwrap_or_default(),
    }
}

fn first_non_empty_string_attribute(
    attributes: &BTreeMap<String, Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .filter_map(|key| string_attribute(attributes, key))
        .find(|value| !value.is_empty())
}

fn string_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bool_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    attributes.get(key).and_then(Value::as_bool)
}

fn f32_attribute(attributes: &BTreeMap<String, Value>, key: &str) -> Option<f32> {
    attributes.get(key).and_then(|value| {
        value
            .as_float()
            .map(|value| value as f32)
            .or_else(|| value.as_integer().map(|value| value as f32))
    })
}
