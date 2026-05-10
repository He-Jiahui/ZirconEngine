use toml::Value;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

pub(super) const TEXT_KEYS: &[&str] = &["text", "label", "value"];
pub(super) const ALT_KEYS: &[&str] = &["accessibility_label", "alt", "alt_text", "icon_alt"];
pub(super) const TOOLTIP_KEYS: &[&str] = &["tooltip"];

pub(super) fn own_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    first_string_attribute(metadata, TEXT_KEYS)
}

pub(super) fn alt_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    first_string_attribute(metadata, ALT_KEYS)
}

pub(super) fn tooltip_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    first_string_attribute(metadata, TOOLTIP_KEYS)
}

pub(super) fn first_string_attribute(
    metadata: Option<&UiTemplateNodeMetadata>,
    keys: &[&str],
) -> Option<String> {
    let metadata = metadata?;
    keys.iter()
        .filter_map(|key| metadata.attributes.get(*key))
        .find_map(scalar_text)
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Float(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}
