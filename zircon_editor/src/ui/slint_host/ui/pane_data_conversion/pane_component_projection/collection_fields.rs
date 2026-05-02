use std::collections::BTreeMap;

use crate::ui::slint_host as host_contract;
use crate::ui::template_runtime::SlintUiHostBindingProjection;
use zircon_runtime_interface::ui::component::{UiValue, UiValueKind};

use super::super::pane_value_conversion::value_as_string;
use super::showcase_actions::showcase_binding_id_for_suffix;

pub(super) fn collection_fields_for_component(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[SlintUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    match component {
        "ArrayField" => array_collection_fields(attributes, bindings),
        "MapField" => map_collection_fields(attributes, bindings),
        _ => Vec::new(),
    }
}

fn array_collection_fields(
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[SlintUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    let element_type = attributes
        .get("element_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Element".to_string());
    let edit_action_id = showcase_binding_id_for_suffix(bindings, "ArrayFieldSetElement");
    let remove_action_id = showcase_binding_id_for_suffix(bindings, "ArrayFieldRemoveElement");
    let move_action_id = showcase_binding_id_for_suffix(bindings, "ArrayFieldMoveElement");
    let items = attributes.get("items").map(UiValue::from_toml);
    let Some(UiValue::Array(values)) = items else {
        return vec![empty_collection_field(
            "array-empty",
            "",
            "",
            element_type.as_str(),
            format!("Empty {element_type} list"),
        )];
    };
    if values.is_empty() {
        return vec![empty_collection_field(
            "array-empty",
            "",
            "",
            element_type.as_str(),
            format!("Empty {element_type} list"),
        )];
    }
    let value_count = values.len();
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let validation = collection_value_validation(&element_type, &value, "array element");
            host_contract::TemplatePaneCollectionFieldData {
                row_id: format!("array-{index}").into(),
                index_text: format!("#{index}").into(),
                key_type: "".into(),
                key_component_role: "".into(),
                key_text: "".into(),
                value_type: element_type.clone().into(),
                value_component_role: collection_field_role(&element_type, Some(&value)).into(),
                value_text: value.display_text().into(),
                value_checked: collection_field_checked(&value),
                validation_level: validation.level.into(),
                validation_message: validation.message.into(),
                key_edit_action_id: "".into(),
                edit_action_id: edit_action_id.clone().into(),
                remove_action_id: remove_action_id.clone().into(),
                move_up_action_id: if index > 0 {
                    move_action_id.clone().into()
                } else {
                    "".into()
                },
                move_up_payload: if index > 0 {
                    format!("array-{index}={}", index - 1).into()
                } else {
                    "".into()
                },
                move_down_action_id: if index + 1 < value_count {
                    move_action_id.clone().into()
                } else {
                    "".into()
                },
                move_down_payload: if index + 1 < value_count {
                    format!("array-{index}={}", index + 1).into()
                } else {
                    "".into()
                },
                empty: false,
            }
        })
        .collect()
}

fn map_collection_fields(
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[SlintUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    let key_type = attributes
        .get("key_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Key".to_string());
    let value_type = attributes
        .get("value_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Value".to_string());
    let edit_action_id = showcase_binding_id_for_suffix(bindings, "MapFieldSetEntry");
    let remove_action_id = showcase_binding_id_for_suffix(bindings, "MapFieldRemoveEntry");
    let entries = attributes.get("entries").map(UiValue::from_toml);
    let Some(UiValue::Map(values)) = entries else {
        return vec![empty_collection_field(
            "map-empty",
            key_type.as_str(),
            "",
            value_type.as_str(),
            format!("Empty {key_type} -> {value_type} map"),
        )];
    };
    if values.is_empty() {
        return vec![empty_collection_field(
            "map-empty",
            key_type.as_str(),
            "",
            value_type.as_str(),
            format!("Empty {key_type} -> {value_type} map"),
        )];
    }
    values
        .into_iter()
        .map(|(key, value)| {
            let validation = collection_map_entry_validation(&key_type, &key, &value_type, &value);
            host_contract::TemplatePaneCollectionFieldData {
                row_id: format!("map-{key}").into(),
                index_text: "".into(),
                key_type: key_type.clone().into(),
                key_component_role: collection_field_role(&key_type, None).into(),
                key_text: key.into(),
                value_type: value_type.clone().into(),
                value_component_role: collection_field_role(&value_type, Some(&value)).into(),
                value_text: value.display_text().into(),
                value_checked: collection_field_checked(&value),
                validation_level: validation.level.into(),
                validation_message: validation.message.into(),
                key_edit_action_id: edit_action_id.clone().into(),
                edit_action_id: edit_action_id.clone().into(),
                remove_action_id: remove_action_id.clone().into(),
                move_up_action_id: "".into(),
                move_up_payload: "".into(),
                move_down_action_id: "".into(),
                move_down_payload: "".into(),
                empty: false,
            }
        })
        .collect()
}

fn empty_collection_field(
    row_id: &str,
    key_type: &str,
    key_text: &str,
    value_type: &str,
    message: String,
) -> host_contract::TemplatePaneCollectionFieldData {
    host_contract::TemplatePaneCollectionFieldData {
        row_id: row_id.into(),
        index_text: "".into(),
        key_type: key_type.into(),
        key_component_role: collection_field_role(key_type, None).into(),
        key_text: key_text.into(),
        value_type: value_type.into(),
        value_component_role: collection_field_role(value_type, None).into(),
        value_text: "".into(),
        value_checked: false,
        validation_level: "warning".into(),
        validation_message: message.into(),
        key_edit_action_id: "".into(),
        edit_action_id: "".into(),
        remove_action_id: "".into(),
        move_up_action_id: "".into(),
        move_up_payload: "".into(),
        move_down_action_id: "".into(),
        move_down_payload: "".into(),
        empty: true,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CollectionFieldValidation {
    level: &'static str,
    message: String,
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

fn collection_map_entry_validation(
    key_type: &str,
    key: &str,
    value_type: &str,
    value: &UiValue,
) -> CollectionFieldValidation {
    let key_validation = collection_key_validation(key_type, key);
    if key_validation.level == "error" {
        return key_validation;
    }
    let value_validation = collection_value_validation(value_type, value, "map value");
    if value_validation.level != "normal" {
        return value_validation;
    }
    key_validation
}

fn collection_key_validation(key_type: &str, key: &str) -> CollectionFieldValidation {
    if key.trim().is_empty() {
        return CollectionFieldValidation::error("Key is required");
    }
    let declared_type = key_type.to_ascii_lowercase();
    if collection_type_is_generic(&declared_type) || declared_type.contains("string") {
        return CollectionFieldValidation::normal();
    }
    if collection_type_is_numeric(&declared_type) && key.parse::<f64>().is_err() {
        return CollectionFieldValidation::error(format!("Expected {key_type} key"));
    }
    if declared_type.contains("bool") && !matches!(key, "true" | "false") {
        return CollectionFieldValidation::error(format!("Expected {key_type} key"));
    }
    CollectionFieldValidation::normal()
}

fn collection_value_validation(
    declared_type: &str,
    value: &UiValue,
    label: &str,
) -> CollectionFieldValidation {
    let normalized_type = declared_type.to_ascii_lowercase();
    if collection_type_is_generic(&normalized_type) {
        return CollectionFieldValidation::normal();
    }
    if collection_type_is_reference_like(&normalized_type) {
        let display = value.display_text();
        if matches!(value, UiValue::Null) || display.trim().is_empty() {
            return CollectionFieldValidation::warning(format!("Missing {declared_type} {label}"));
        }
        return CollectionFieldValidation::normal();
    }
    if normalized_type.contains("bool") && !matches!(value, UiValue::Bool(_)) {
        return CollectionFieldValidation::error(format!("Expected bool {label}"));
    }
    if collection_type_is_numeric(&normalized_type) && value.as_f64().is_none() {
        return CollectionFieldValidation::error(format!("Expected numeric {label}"));
    }
    if normalized_type.contains("color") && !collection_value_is_color(value) {
        return CollectionFieldValidation::error(format!("Expected color {label}"));
    }
    if normalized_type.contains("vec2") && !collection_value_is_vector(value, 2) {
        return CollectionFieldValidation::error(format!("Expected Vec2 {label}"));
    }
    if normalized_type.contains("vec3") && !collection_value_is_vector(value, 3) {
        return CollectionFieldValidation::error(format!("Expected Vec3 {label}"));
    }
    if normalized_type.contains("vec4") && !collection_value_is_vector(value, 4) {
        return CollectionFieldValidation::error(format!("Expected Vec4 {label}"));
    }
    CollectionFieldValidation::normal()
}

fn collection_type_is_generic(normalized_type: &str) -> bool {
    normalized_type.is_empty()
        || matches!(
            normalized_type,
            "any" | "value" | "uivalue" | "variant" | "unknown"
        )
}

fn collection_type_is_numeric(normalized_type: &str) -> bool {
    normalized_type.contains("int")
        || normalized_type.contains("float")
        || normalized_type.contains("double")
        || normalized_type.contains("number")
}

fn collection_type_is_reference_like(normalized_type: &str) -> bool {
    normalized_type.contains("asset")
        || normalized_type.contains("instance")
        || normalized_type.contains("object")
        || normalized_type.contains("ref")
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

fn collection_field_role(declared_type: &str, value: Option<&UiValue>) -> &'static str {
    let declared_type = declared_type.to_ascii_lowercase();
    if declared_type.contains("bool") {
        return "checkbox";
    }
    if declared_type.contains("asset") {
        return "asset-field";
    }
    if declared_type.contains("instance") || declared_type.contains("object") {
        return "object-field";
    }
    if declared_type.contains("color") {
        return "color-field";
    }
    if declared_type.contains("vec") || declared_type.contains("vector") {
        return "vector-field";
    }
    if declared_type.contains("int")
        || declared_type.contains("float")
        || declared_type.contains("double")
        || declared_type.contains("number")
    {
        return "number-field";
    }
    if declared_type.contains("ref") {
        return "reference-field";
    }

    match value.map(UiValue::kind) {
        Some(UiValueKind::Bool) => "checkbox",
        Some(UiValueKind::Int) | Some(UiValueKind::Float) => "number-field",
        Some(UiValueKind::Color) => "color-field",
        Some(UiValueKind::Vec2) | Some(UiValueKind::Vec3) | Some(UiValueKind::Vec4) => {
            "vector-field"
        }
        Some(UiValueKind::AssetRef) => "asset-field",
        Some(UiValueKind::InstanceRef) => "object-field",
        _ => "text-field",
    }
}

fn collection_field_checked(value: &UiValue) -> bool {
    matches!(value, UiValue::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_value(value: &str) -> toml::Value {
        toml::Value::String(value.to_string())
    }

    #[test]
    fn collection_fields_mark_empty_arrays_as_warning_rows() {
        let mut attributes = BTreeMap::new();
        attributes.insert("element_type".to_string(), string_value("Float"));
        attributes.insert("items".to_string(), toml::Value::Array(Vec::new()));

        let fields = array_collection_fields(&attributes, &[]);

        assert_eq!(fields.len(), 1);
        let field = &fields[0];
        assert!(field.empty);
        assert_eq!(field.validation_level.as_str(), "warning");
        assert_eq!(field.validation_message.as_str(), "Empty Float list");
        assert_eq!(field.value_component_role.as_str(), "number-field");
    }

    #[test]
    fn collection_fields_validate_map_keys_and_values_per_row() {
        let mut entries = toml::map::Map::new();
        entries.insert("bad".to_string(), string_value("fast"));
        entries.insert("visible".to_string(), toml::Value::Boolean(true));
        let mut attributes = BTreeMap::new();
        attributes.insert("key_type".to_string(), string_value("String"));
        attributes.insert("value_type".to_string(), string_value("Float"));
        attributes.insert("entries".to_string(), toml::Value::Table(entries));

        let fields = map_collection_fields(&attributes, &[]);

        let bad_value = fields
            .iter()
            .find(|field| field.key_text.as_str() == "bad")
            .expect("bad map row should be projected");
        assert_eq!(bad_value.validation_level.as_str(), "error");
        assert_eq!(
            bad_value.validation_message.as_str(),
            "Expected numeric map value"
        );

        let bool_value = fields
            .iter()
            .find(|field| field.key_text.as_str() == "visible")
            .expect("visible map row should be projected");
        assert_eq!(bool_value.validation_level.as_str(), "error");
        assert_eq!(
            bool_value.validation_message.as_str(),
            "Expected numeric map value"
        );
    }

    #[test]
    fn collection_fields_validate_non_string_key_types() {
        let mut entries = toml::map::Map::new();
        entries.insert("speed".to_string(), toml::Value::Float(1.0));
        let mut attributes = BTreeMap::new();
        attributes.insert("key_type".to_string(), string_value("Int"));
        attributes.insert("value_type".to_string(), string_value("Float"));
        attributes.insert("entries".to_string(), toml::Value::Table(entries));

        let fields = map_collection_fields(&attributes, &[]);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].validation_level.as_str(), "error");
        assert_eq!(fields[0].validation_message.as_str(), "Expected Int key");
    }
}
