use std::cmp::Ordering;

use zircon_runtime_interface::ui::{component::UiValue, tree::UiTemplateNodeMetadata};

use super::{
    bool_attribute, explicit_false_attribute, role_is_one_of, string_attribute, toml_number,
    UiTableColumnResizeDragToken,
};

const TABLE_COLUMN_RESIZE_DRAG_PREFIX: &str = "table_column_resize";
const DEFAULT_MIN_COLUMN_WIDTH: f64 = 40.0;
const TABLE_COLUMN_RESIZE_HANDLE_ROLES: [&str; 3] = [
    "table-column-resize-handle",
    "data-grid-column-resize-handle",
    "column-resize-handle",
];
const TABLE_COLUMN_SORT_HEADER_ROLES: [&str; 4] = [
    "table-column-header",
    "table-sort-header",
    "data-grid-column-header",
    "column-header",
];
const TABLE_COLUMN_FIELD_PROPERTIES: [&str; 8] = [
    "field",
    "column",
    "column_id",
    "columnId",
    "id",
    "key",
    "name",
    "property",
];

pub(super) fn is_table_column_resize_handle(metadata: &UiTemplateNodeMetadata) -> bool {
    role_is_one_of(metadata, &TABLE_COLUMN_RESIZE_HANDLE_ROLES)
}

pub(super) fn is_table_column_sort_header(metadata: &UiTemplateNodeMetadata) -> bool {
    role_is_one_of(metadata, &TABLE_COLUMN_SORT_HEADER_ROLES)
}

pub(super) fn table_column_resize_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "disableColumnResize")
        || bool_attribute(metadata, "disable_column_resize")
        || explicit_false_attribute(metadata, "column_resizing")
        || explicit_false_attribute(metadata, "columnResizing")
        || explicit_false_attribute(metadata, "resizable_columns")
        || explicit_false_attribute(metadata, "resizableColumns")
}

pub(super) fn table_sorting_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "disableColumnSorting")
        || bool_attribute(metadata, "disable_column_sorting")
        || explicit_false_attribute(metadata, "sorting")
        || explicit_false_attribute(metadata, "column_sorting")
        || explicit_false_attribute(metadata, "columnSorting")
        || explicit_false_attribute(metadata, "sortable_columns")
        || explicit_false_attribute(metadata, "sortableColumns")
}

pub(super) fn table_column_sorting_disabled(
    header_metadata: &UiTemplateNodeMetadata,
    owner_metadata: &UiTemplateNodeMetadata,
    field: &str,
) -> bool {
    explicit_false_attribute(header_metadata, "sortable")
        || explicit_false_attribute(header_metadata, "column_sorting")
        || explicit_false_attribute(header_metadata, "columnSorting")
        || owner_metadata
            .attributes
            .get("columns")
            .and_then(toml::Value::as_array)
            .and_then(|columns| {
                columns.iter().find_map(|column| {
                    let column = column.as_table()?;
                    toml_column_matches(column, field)
                        .then(|| column.get("sortable").and_then(toml::Value::as_bool))?
                })
            })
            == Some(false)
}

pub(super) fn table_column_field(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    TABLE_COLUMN_FIELD_PROPERTIES
        .iter()
        .find_map(|property| string_attribute(metadata, property))
        .filter(|field| !field.is_empty())
}

pub(super) fn table_column_width(metadata: &UiTemplateNodeMetadata, field: &str) -> Option<f64> {
    metadata
        .attributes
        .get("column_widths")
        .and_then(toml::Value::as_table)
        .and_then(|widths| widths.get(field))
        .and_then(toml_number)
        .or_else(|| {
            metadata
                .attributes
                .get("columns")
                .and_then(toml::Value::as_array)
                .and_then(|columns| {
                    columns.iter().find_map(|column| {
                        let column = column.as_table()?;
                        if toml_column_matches(column, field) {
                            column.get("width").and_then(toml_number)
                        } else {
                            None
                        }
                    })
                })
        })
}

pub(super) fn next_table_sort_direction(
    metadata: &UiTemplateNodeMetadata,
    field: &str,
) -> &'static str {
    if table_sort_column(metadata).as_deref() != Some(field) {
        return "asc";
    }
    match table_sort_direction(metadata) {
        Some("asc") => "desc",
        Some("desc") => "asc",
        _ => "asc",
    }
}

pub(super) fn table_uses_client_sorting(metadata: &UiTemplateNodeMetadata) -> bool {
    !matches!(
        string_attribute(metadata, "sortingMode").as_deref(),
        Some("server")
    )
}

pub(super) fn compare_table_row_value(left: &UiValue, right: &UiValue, field: &str) -> Ordering {
    let left = table_row_field(left, field);
    let right = table_row_field(right, field);
    match (
        left.and_then(UiValue::as_f64),
        right.and_then(UiValue::as_f64),
    ) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => match (
            left.and_then(borrowed_sort_text),
            right.and_then(borrowed_sort_text),
        ) {
            (Some(left), Some(right)) => left.cmp(right),
            _ => left
                .map(UiValue::display_text)
                .unwrap_or_default()
                .cmp(&right.map(UiValue::display_text).unwrap_or_default()),
        },
    }
}

fn borrowed_sort_text(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value)
        | UiValue::Color(value)
        | UiValue::AssetRef(value)
        | UiValue::InstanceRef(value)
        | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

pub(super) fn table_min_column_width(metadata: &UiTemplateNodeMetadata, field: &str) -> f64 {
    metadata
        .attributes
        .get("columns")
        .and_then(toml::Value::as_array)
        .and_then(|columns| {
            columns.iter().find_map(|column| {
                let column = column.as_table()?;
                if toml_column_matches(column, field) {
                    column.get("minWidth").and_then(toml_number)
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            metadata
                .attributes
                .get("min_column_width")
                .and_then(toml_number)
        })
        .or_else(|| {
            metadata
                .attributes
                .get("minColumnWidth")
                .and_then(toml_number)
        })
        .unwrap_or(DEFAULT_MIN_COLUMN_WIDTH)
}

pub(super) fn table_column_matches(
    column: &std::collections::BTreeMap<String, UiValue>,
    field: &str,
) -> bool {
    TABLE_COLUMN_FIELD_PROPERTIES.iter().any(|property| {
        matches!(
            column.get(*property),
            Some(UiValue::String(value) | UiValue::Enum(value)) if value == field
        )
    })
}

pub(super) fn column_width_payload(field: &str, width: f64) -> UiValue {
    UiValue::Map(std::collections::BTreeMap::from([
        ("field".to_string(), UiValue::String(field.to_string())),
        ("width".to_string(), UiValue::Float(width)),
    ]))
}

pub(super) fn encode_table_column_resize_drag(
    start_width: f64,
    min_width: f64,
    field: &str,
) -> String {
    format!("{TABLE_COLUMN_RESIZE_DRAG_PREFIX}:{start_width}:{min_width}:{field}")
}

pub(super) fn decode_table_column_resize_drag(value: &str) -> Option<UiTableColumnResizeDragToken> {
    let rest = value.strip_prefix(TABLE_COLUMN_RESIZE_DRAG_PREFIX)?;
    let rest = rest.strip_prefix(':')?;
    let mut parts = rest.splitn(3, ':');
    let start_width = parts.next()?.parse::<f64>().ok()?;
    let min_width = parts.next()?.parse::<f64>().ok()?;
    let field = parts.next()?.to_string();
    Some(UiTableColumnResizeDragToken {
        field,
        start_width,
        min_width,
    })
}

fn table_sort_column(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "sort_column")
        .or_else(|| string_attribute(metadata, "sortField"))
        .or_else(|| string_attribute(metadata, "sort_field"))
        .or_else(|| {
            metadata
                .attributes
                .get("sortModel")
                .and_then(toml::Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(toml::Value::as_table)
                .and_then(|entry| entry.get("field"))
                .and_then(toml::Value::as_str)
                .map(str::to_string)
        })
}

fn table_sort_direction(metadata: &UiTemplateNodeMetadata) -> Option<&'static str> {
    string_attribute(metadata, "sort_direction")
        .or_else(|| string_attribute(metadata, "sortDirection"))
        .and_then(|direction| normalize_sort_direction(&direction))
        .or_else(|| {
            metadata
                .attributes
                .get("sortModel")
                .and_then(toml::Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(toml::Value::as_table)
                .and_then(|entry| entry.get("sort"))
                .and_then(toml::Value::as_str)
                .and_then(normalize_sort_direction)
        })
}

fn normalize_sort_direction(value: &str) -> Option<&'static str> {
    match value {
        "asc" | "ascending" => Some("asc"),
        "desc" | "descending" => Some("desc"),
        "none" | "" => Some("none"),
        _ => None,
    }
}

fn table_row_field<'a>(row: &'a UiValue, field: &str) -> Option<&'a UiValue> {
    match row {
        UiValue::Map(values) => values.get(field),
        _ => None,
    }
}

fn toml_column_matches(column: &toml::map::Map<String, toml::Value>, field: &str) -> bool {
    TABLE_COLUMN_FIELD_PROPERTIES
        .iter()
        .any(|property| matches!(column.get(*property).and_then(toml::Value::as_str), Some(value) if value == field))
}
