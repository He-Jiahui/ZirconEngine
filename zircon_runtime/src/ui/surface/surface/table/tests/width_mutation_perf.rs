use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{component::UiValue, tree::UiTemplateNodeMetadata};

use super::*;

#[test]
fn optimization_batch_20260909_runtime438_width_batch_preserves_projection_shape() {
    let mut widths = toml::map::Map::new();
    widths.insert("name".to_string(), toml::Value::Float(96.0));
    let mut column = toml::map::Map::new();
    column.insert("field".to_string(), toml::Value::String("name".to_string()));
    column.insert("width".to_string(), toml::Value::Float(96.0));
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("column_widths".to_string(), toml::Value::Table(widths)),
            (
                "columns".to_string(),
                toml::Value::Array(vec![toml::Value::Table(column)]),
            ),
        ]),
        ..UiTemplateNodeMetadata::default()
    };

    let values = table_column_width_values(&metadata, "name", 144.0);
    assert_eq!(values.len(), 2);
    assert!(
        matches!(values[0].1, UiValue::Map(ref values) if values["name"] == UiValue::Float(144.0))
    );
    assert!(
        matches!(values[1].1, UiValue::Array(ref values) if matches!(&values[0], UiValue::Map(column) if column["width"] == UiValue::Float(144.0)))
    );
}

#[test]
fn optimization_batch_20260909_runtime438_width_batch_keeps_map_when_columns_are_missing() {
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([(
            "column_widths".to_string(),
            toml::Value::Table(toml::map::Map::new()),
        )]),
        ..UiTemplateNodeMetadata::default()
    };
    let values = table_column_width_values(&metadata, "name", 144.0);
    assert_eq!(values.len(), 1);
    assert!(
        matches!(values[0].1, UiValue::Map(ref values) if values["name"] == UiValue::Float(144.0))
    );
}
