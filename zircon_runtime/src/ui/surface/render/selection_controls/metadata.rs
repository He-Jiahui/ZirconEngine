use toml::Value;
use zircon_runtime_interface::ui::{
    style::{UiPainterFamily, UiRgbaColor},
    tree::UiTemplateNodeMetadata,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectionControlKind {
    Checkbox,
    Radio,
    Toggle,
}

pub(super) fn selection_control_kind(
    metadata: &UiTemplateNodeMetadata,
) -> Option<SelectionControlKind> {
    match metadata.component.as_str() {
        "Checkbox" => Some(SelectionControlKind::Checkbox),
        "Radio" => Some(SelectionControlKind::Radio),
        "Toggle" | "Switch" => Some(SelectionControlKind::Toggle),
        _ => None,
    }
}

pub(super) fn selection_painter_family(metadata: &UiTemplateNodeMetadata) -> UiPainterFamily {
    match metadata.component.as_str() {
        "Checkbox" => UiPainterFamily::Checkbox,
        "Radio" => UiPainterFamily::Radio,
        "Toggle" | "Switch" => UiPainterFamily::Toggle,
        _ => UiPainterFamily::Generic,
    }
}

pub(super) fn control_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "text", "value_text"]
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
    let (red, green, blue, alpha) = match encoded.len() {
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
    Some(UiRgbaColor::from_u8(red, green, blue, alpha))
}
