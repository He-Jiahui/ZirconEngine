use std::{cmp::Ordering, collections::BTreeMap};

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

pub(super) fn apply_value_event(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Result<bool, UiComponentEventError> {
    if !is_table_family(descriptor) {
        return Ok(false);
    }

    match property {
        "sort_column" | "sortField" | "sort_field" => {
            let Some(column) = string_value(value) else {
                return Ok(false);
            };
            apply_sort_column(state, descriptor, &column);
            Ok(true)
        }
        "sort_direction" | "sortDirection" => {
            let Some(direction) =
                string_value(value).and_then(|value| normalize_sort_direction(&value))
            else {
                return Ok(false);
            };
            let Some(column) = string_setting(state, "sort_column") else {
                return Ok(false);
            };
            apply_sort(state, descriptor, &column, direction);
            Ok(true)
        }
        "column_width" | "columnWidth" => {
            let Some((field, width)) = column_width_payload(value) else {
                return Ok(false);
            };
            apply_column_width(state, &field, width);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn is_table_family(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "table" | "data-grid" | "mui-x-data-grid"
    ) || matches!(descriptor.id.as_str(), "Table" | "DataGrid")
}

fn apply_sort_column(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    column: &str,
) {
    if column.is_empty() {
        apply_sort(state, descriptor, "", "none");
        return;
    }

    let current_column = string_setting(state, "sort_column");
    let current_direction =
        string_setting(state, "sort_direction").and_then(|value| normalize_sort_direction(&value));
    let next_direction = if current_column.as_deref() == Some(column) {
        match current_direction {
            Some("asc") => "desc",
            Some("desc") => "asc",
            _ => "asc",
        }
    } else {
        "asc"
    };

    apply_sort(state, descriptor, column, next_direction);
}

fn apply_sort(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    column: &str,
    direction: &str,
) {
    if direction == "none" || column.is_empty() {
        super::set_value(
            state,
            "sort_column".to_string(),
            UiValue::String(String::new()),
        );
        super::set_value(
            state,
            "sort_direction".to_string(),
            UiValue::String("none".to_string()),
        );
        if descriptor.prop("sortModel").is_some() {
            super::set_value(state, "sortModel".to_string(), UiValue::Array(Vec::new()));
        }
        return;
    }

    super::set_value(
        state,
        "sort_column".to_string(),
        UiValue::String(column.to_string()),
    );
    super::set_value(
        state,
        "sort_direction".to_string(),
        UiValue::String(direction.to_string()),
    );
    if descriptor.prop("sortModel").is_some() {
        super::set_value(
            state,
            "sortModel".to_string(),
            UiValue::Array(vec![UiValue::Map(sort_model_entry(column, direction))]),
        );
    }
    if table_uses_client_sorting(state) {
        sort_rows(state, column, direction);
    }
}

fn sort_model_entry(column: &str, direction: &str) -> BTreeMap<String, UiValue> {
    let mut entry = BTreeMap::new();
    entry.insert("field".to_string(), UiValue::String(column.to_string()));
    entry.insert("sort".to_string(), UiValue::String(direction.to_string()));
    entry
}

fn table_uses_client_sorting(state: &UiComponentState) -> bool {
    !matches!(
        string_setting(state, "sortingMode").as_deref(),
        Some("server")
    )
}

fn sort_rows(state: &mut UiComponentState, column: &str, direction: &str) {
    let Some(UiValue::Array(rows)) = state.values.get_mut("rows") else {
        return;
    };
    rows.sort_by(|left, right| compare_row_value(left, right, column));
    if direction == "desc" {
        rows.reverse();
    }
}

fn compare_row_value(left: &UiValue, right: &UiValue, column: &str) -> Ordering {
    let left = row_field(left, column);
    let right = row_field(right, column);
    match (
        left.and_then(UiValue::as_f64),
        right.and_then(UiValue::as_f64),
    ) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => left
            .map(UiValue::display_text)
            .unwrap_or_default()
            .cmp(&right.map(UiValue::display_text).unwrap_or_default()),
    }
}

fn row_field<'a>(row: &'a UiValue, column: &str) -> Option<&'a UiValue> {
    match row {
        UiValue::Map(values) => values.get(column),
        _ => None,
    }
}

fn apply_column_width(state: &mut UiComponentState, field: &str, width: f64) {
    let mut widths = match state.values.get("column_widths") {
        Some(UiValue::Map(widths)) => widths.clone(),
        _ => BTreeMap::new(),
    };
    widths.insert(field.to_string(), UiValue::Float(width));
    super::set_value(state, "column_widths".to_string(), UiValue::Map(widths));

    let Some(UiValue::Array(columns)) = state.values.get_mut("columns") else {
        return;
    };
    for column in columns {
        let UiValue::Map(values) = column else {
            continue;
        };
        if column_matches(values, field) {
            values.insert("width".to_string(), UiValue::Float(width));
            break;
        }
    }
}

fn column_matches(column: &BTreeMap<String, UiValue>, field: &str) -> bool {
    ["field", "id", "key", "name"].iter().any(|property| {
        matches!(
            column.get(*property),
            Some(UiValue::String(value) | UiValue::Enum(value)) if value == field
        )
    })
}

fn column_width_payload(value: &UiValue) -> Option<(String, f64)> {
    let UiValue::Map(payload) = value else {
        return None;
    };
    let field = ["field", "column", "column_id", "columnId", "id"]
        .iter()
        .find_map(|property| string_value(payload.get(*property)?))?;
    let width = ["width", "value"]
        .iter()
        .find_map(|property| payload.get(*property)?.as_f64())?;
    Some((field, width))
}

fn string_setting(state: &UiComponentState, property: &str) -> Option<String> {
    state.values.get(property).and_then(string_value)
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn normalize_sort_direction(value: &str) -> Option<&'static str> {
    match value {
        "asc" | "ascending" => Some("asc"),
        "desc" | "descending" => Some("desc"),
        "none" | "" => Some("none"),
        _ => None,
    }
}
