use std::collections::BTreeMap;

use super::{array, map};

fn string_value(value: &str) -> toml::Value {
    toml::Value::String(value.to_string())
}

#[test]
fn collection_fields_mark_empty_arrays_as_warning_rows() {
    let mut attributes = BTreeMap::new();
    attributes.insert("element_type".to_string(), string_value("Float"));
    attributes.insert("items".to_string(), toml::Value::Array(Vec::new()));

    let fields = array::array_collection_fields(&attributes, &[]);

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

    let fields = map::map_collection_fields(&attributes, &[]);

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

    let fields = map::map_collection_fields(&attributes, &[]);

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].validation_level.as_str(), "error");
    assert_eq!(fields[0].validation_message.as_str(), "Expected Int key");
}
