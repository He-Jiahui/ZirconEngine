use toml::Value;
use zircon_runtime_interface::ui::{style::UiRgbaColor, tree::UiTemplateNodeMetadata};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SegmentedControlKind {
    SegmentedControl,
    Tab,
}

pub(super) fn control_kind(metadata: &UiTemplateNodeMetadata) -> Option<SegmentedControlKind> {
    match metadata.component.as_str() {
        "SegmentedControl" | "Segmented" => Some(SegmentedControlKind::SegmentedControl),
        "Tab" | "PanelTab" => Some(SegmentedControlKind::Tab),
        _ => None,
    }
}

pub(super) fn is_segmented_or_tab(metadata: &UiTemplateNodeMetadata) -> bool {
    control_kind(metadata).is_some()
}

pub(super) fn segmented_options(metadata: &UiTemplateNodeMetadata) -> Vec<String> {
    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .map(|options| {
            options
                .iter()
                .filter_map(option_string)
                .filter(|option| !option.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn option_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.to_string()),
        Value::Table(table) => ["label", "text", "value", "id", "name"]
            .iter()
            .find_map(|key| table.get(*key))
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

pub(super) fn selected_segment_value(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    ["value", "value_text", "selected", "text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn option_is_selected(option: &str, selected: Option<&str>) -> bool {
    selected.is_some_and(|selected| option.trim().eq_ignore_ascii_case(selected))
}

pub(super) fn option_label(option: &str) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => {
            let mut label = first.to_ascii_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}

pub(super) fn group_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "label_text", "group_label"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

pub(super) fn tab_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["text", "label", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

pub(super) fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

pub(super) fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

pub(super) fn first_rgba_attribute(
    metadata: &UiTemplateNodeMetadata,
    keys: &[&str],
) -> Option<UiRgbaColor> {
    keys.iter().find_map(|key| {
        metadata
            .style_overrides
            .get(*key)
            .or_else(|| metadata.attributes.get(*key))
            .and_then(Value::as_str)
            .and_then(parse_css_color)
    })
}

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
}

pub(super) fn line_height(
    metadata: &UiTemplateNodeMetadata,
    absolute_key: &str,
    ratio_key: &str,
    font_size: f32,
    default: f32,
) -> f32 {
    metric_attribute(metadata, absolute_key)
        .filter(|value| *value > 0.0)
        .or_else(|| {
            metric_attribute(metadata, ratio_key)
                .filter(|value| *value > 0.0)
                .map(|ratio| font_size * ratio)
        })
        .unwrap_or(default)
}

fn parse_css_color(value: &str) -> Option<UiRgbaColor> {
    let encoded = value.trim().strip_prefix('#')?;
    if !encoded.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let (r, g, b, a) = match encoded.len() {
        6 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::MAX,
        ),
        8 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::from_str_radix(&encoded[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(r, g, b, a))
}
