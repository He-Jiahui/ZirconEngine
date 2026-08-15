use toml::Value;
use zircon_runtime_interface::ui::{surface::UiTextAlign, tree::UiTemplateNodeMetadata};

pub(super) fn value_to_display_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        _ => value.to_string(),
    }
}

pub(super) fn string_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(super) fn string_array_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Vec<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}

pub(super) fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

pub(super) fn integer_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<i32> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_integer)
        .map(|value| value as i32)
}

pub(super) fn text_align_name(align: UiTextAlign) -> &'static str {
    match align {
        UiTextAlign::Left => "left",
        UiTextAlign::Center => "center",
        UiTextAlign::Right => "right",
        UiTextAlign::Start => "start",
        UiTextAlign::End => "end",
        UiTextAlign::Justify => "justify",
    }
}
