use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::collection_window::collect_visible_collection_items;
use super::virtualization::ProjectedVirtualization;
use crate::ui::retained_host as host_contract;
use toml::Value;

pub(super) fn projected_collection_items(
    attributes: &BTreeMap<String, toml::Value>,
    virtualization: &ProjectedVirtualization,
) -> Vec<String> {
    let Some(values) = attributes
        .get("collection_items")
        .and_then(toml::Value::as_array)
    else {
        return Vec::new();
    };
    let items = values.iter().filter_map(value_as_string);

    if virtualization.enabled {
        collect_visible_collection_items(
            items,
            virtualization.visible_start,
            virtualization.visible_count,
            virtualization.overscan,
        )
    } else {
        items.collect()
    }
}

pub(super) fn projected_collection_rows(
    attributes: &BTreeMap<String, Value>,
    virtualization: &ProjectedVirtualization,
) -> Vec<host_contract::TemplatePaneCollectionRowData> {
    let Some(identity_field) = attributes
        .get("row_identity_field")
        .and_then(Value::as_str)
        .filter(|field| !field.is_empty())
    else {
        return Vec::new();
    };
    let rows = attributes
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(source_index, row)| {
            let row = row.as_table()?;
            let identity = row.get(identity_field)?;
            let (identity_kind, identity_text) = scalar_identity(identity)?;
            Some(host_contract::TemplatePaneCollectionRowData {
                source_index: i32::try_from(source_index).unwrap_or(i32::MAX),
                row_identity_field: identity_field.into(),
                identity_kind: identity_kind.into(),
                identity_text: identity_text.clone().into(),
                label: row
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or(identity_text.as_str())
                    .into(),
            })
        });

    if virtualization.enabled {
        collect_visible_collection_items(
            rows,
            virtualization.visible_start,
            virtualization.visible_count,
            virtualization.overscan,
        )
    } else {
        rows.collect()
    }
}

fn scalar_identity(value: &Value) -> Option<(&'static str, String)> {
    match value {
        Value::String(value) => Some(("string", value.clone())),
        Value::Integer(value) => Some(("integer", value.to_string())),
        Value::Float(value) => Some(("float", value.to_string())),
        Value::Boolean(value) => Some(("boolean", value.to_string())),
        _ => None,
    }
}
