use zircon_runtime_interface::ui::component::UiValue;

use super::type_tokens::CollectionTypeTraits;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CollectionFieldValidation {
    pub(super) level: &'static str,
    pub(super) message: String,
}

impl CollectionFieldValidation {
    fn normal() -> Self {
        Self {
            level: "normal",
            message: String::new(),
        }
    }

    fn warning(message: impl Into<String>) -> Self {
        Self {
            level: "warning",
            message: message.into(),
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            level: "error",
            message: message.into(),
        }
    }
}

pub(super) fn collection_map_entry_validation(
    key_type: &str,
    key_traits: CollectionTypeTraits,
    key: &str,
    value_type: &str,
    value_traits: CollectionTypeTraits,
    value: &UiValue,
) -> CollectionFieldValidation {
    let key_validation = collection_key_validation(key_type, key_traits, key);
    if key_validation.level == "error" {
        return key_validation;
    }
    let value_validation =
        collection_value_validation(value_type, value_traits, value, "map value");
    if value_validation.level != "normal" {
        return value_validation;
    }
    key_validation
}

fn collection_key_validation(
    key_type: &str,
    traits: CollectionTypeTraits,
    key: &str,
) -> CollectionFieldValidation {
    if key.trim().is_empty() {
        return CollectionFieldValidation::error("Key is required");
    }
    if traits.is_generic() || traits.is_string() {
        return CollectionFieldValidation::normal();
    }
    if traits.is_numeric() && key.parse::<f64>().is_err() {
        return CollectionFieldValidation::error(format!("Expected {key_type} key"));
    }
    if traits.is_boolean() && !matches!(key, "true" | "false") {
        return CollectionFieldValidation::error(format!("Expected {key_type} key"));
    }
    CollectionFieldValidation::normal()
}

pub(super) fn collection_value_validation(
    declared_type: &str,
    traits: CollectionTypeTraits,
    value: &UiValue,
    label: &str,
) -> CollectionFieldValidation {
    if traits.is_generic() {
        return CollectionFieldValidation::normal();
    }
    if traits.is_reference_like() {
        let display = value.display_text();
        if matches!(value, UiValue::Null) || display.trim().is_empty() {
            return CollectionFieldValidation::warning(format!("Missing {declared_type} {label}"));
        }
        return CollectionFieldValidation::normal();
    }
    if traits.is_boolean() && !matches!(value, UiValue::Bool(_)) {
        return CollectionFieldValidation::error(format!("Expected bool {label}"));
    }
    if traits.is_numeric() && value.as_f64().is_none() {
        return CollectionFieldValidation::error(format!("Expected numeric {label}"));
    }
    if traits.is_color() && !collection_value_is_color(value) {
        return CollectionFieldValidation::error(format!("Expected color {label}"));
    }
    if traits.is_vec2() && !collection_value_is_vector(value, 2) {
        return CollectionFieldValidation::error(format!("Expected Vec2 {label}"));
    }
    if traits.is_vec3() && !collection_value_is_vector(value, 3) {
        return CollectionFieldValidation::error(format!("Expected Vec3 {label}"));
    }
    if traits.is_vec4() && !collection_value_is_vector(value, 4) {
        return CollectionFieldValidation::error(format!("Expected Vec4 {label}"));
    }
    CollectionFieldValidation::normal()
}

fn collection_value_is_color(value: &UiValue) -> bool {
    match value {
        UiValue::Color(_) => true,
        UiValue::String(value) => value.starts_with('#') && matches!(value.len(), 7 | 9),
        _ => false,
    }
}

fn collection_value_is_vector(value: &UiValue, component_count: usize) -> bool {
    match (value, component_count) {
        (UiValue::Vec2(_), 2) | (UiValue::Vec3(_), 3) | (UiValue::Vec4(_), 4) => true,
        (UiValue::Array(values), _) => values.len() == component_count,
        _ => false,
    }
}
