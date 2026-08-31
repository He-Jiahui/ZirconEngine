use toml::Value;
use zircon_runtime_interface::ui::{style::UiRgbaColor, tree::UiTemplateNodeMetadata};

#[derive(Clone, Copy)]
pub(super) enum ButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

pub(super) fn is_button_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Button" | "ToggleButton" | "IconButton"
    )
}

pub(super) fn is_icon_button(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "IconButton"
}

pub(super) fn button_kind(metadata: &UiTemplateNodeMetadata) -> ButtonKind {
    let values = [
        string_attribute(metadata, "button_color"),
        string_attribute(metadata, "button_variant"),
        string_attribute(metadata, "validation_level"),
    ];
    if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "danger") || contains_ascii_case(value, "error"))
    {
        ButtonKind::Danger
    } else if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "primary"))
    {
        ButtonKind::Primary
    } else if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "tertiary") || contains_ascii_case(value, "text"))
    {
        ButtonKind::Tertiary
    } else {
        ButtonKind::Secondary
    }
}

fn contains_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

pub(super) fn button_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "text", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

pub(super) fn icon_name(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["icon", "image", "source"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|icon| !icon.is_empty())
        .map(|icon| {
            icon.rsplit(['/', '\\'])
                .next()
                .unwrap_or(icon)
                .trim_end_matches(".svg")
                .to_string()
        })
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
