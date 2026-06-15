use std::collections::BTreeMap;

use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
};

#[test]
fn table_sort_column_toggles_direction_and_sorts_rows() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let table = registry.descriptor("Table").expect("Table descriptor");
    assert!(table.supports_event(UiComponentEventKind::ValueChanged));
    assert!(table.prop("sort_column").is_some());
    assert!(table.prop("sort_direction").is_some());

    let mut state = UiComponentState::new().with_value(
        "rows",
        UiValue::Array(vec![
            table_row("sphere", "Sphere", 200),
            table_row("cube", "Cube", 12),
            table_row("camera", "Camera", 1),
        ]),
    );

    state
        .apply_event(
            table,
            UiComponentEvent::ValueChanged {
                property: "sort_column".to_string(),
                value: UiValue::String("name".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("sort_column"),
        Some(&UiValue::String("name".to_string()))
    );
    assert_eq!(
        state.value("sort_direction"),
        Some(&UiValue::String("asc".to_string()))
    );
    assert_row_ids(&state, ["camera", "cube", "sphere"]);

    state
        .apply_event(
            table,
            UiComponentEvent::ValueChanged {
                property: "sort_column".to_string(),
                value: UiValue::String("name".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("sort_direction"),
        Some(&UiValue::String("desc".to_string()))
    );
    assert_row_ids(&state, ["sphere", "cube", "camera"]);
}

#[test]
fn data_grid_server_sort_updates_sort_model_without_reordering_rows() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert!(data_grid.supports_event(UiComponentEventKind::ValueChanged));
    assert!(data_grid.prop("sortModel").is_some());
    assert!(data_grid.prop("sort_column").is_some());
    assert!(data_grid.prop("sort_direction").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "rows",
            UiValue::Array(vec![
                table_row("imported", "Imported", 4),
                table_row("failed", "Failed", 9),
            ]),
        )
        .with_value("sortingMode", UiValue::String("server".to_string()));

    state
        .apply_event(
            data_grid,
            UiComponentEvent::ValueChanged {
                property: "sort_column".to_string(),
                value: UiValue::String("triangles".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("sort_column"),
        Some(&UiValue::String("triangles".to_string()))
    );
    assert_eq!(
        state.value("sort_direction"),
        Some(&UiValue::String("asc".to_string()))
    );
    assert_eq!(
        state.value("sortModel"),
        Some(&sort_model("triangles", "asc"))
    );
    assert_row_ids(&state, ["imported", "failed"]);
}

#[test]
fn data_grid_column_width_updates_width_map_and_column_entry() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert!(data_grid.supports_event(UiComponentEventKind::ValueChanged));
    assert!(data_grid.prop("column_widths").is_some());

    let mut state = UiComponentState::new().with_value(
        "columns",
        UiValue::Array(vec![
            table_column("name", 160.0),
            table_column("triangles", 96.0),
        ]),
    );

    state
        .apply_event(
            data_grid,
            UiComponentEvent::ValueChanged {
                property: "column_width".to_string(),
                value: column_width_payload("triangles", 132.0),
            },
        )
        .unwrap();

    let mut expected_widths = BTreeMap::new();
    expected_widths.insert("triangles".to_string(), UiValue::Float(132.0));
    assert_eq!(
        state.value("column_widths"),
        Some(&UiValue::Map(expected_widths))
    );
    assert_column_width(&state, "triangles", 132.0);
}

fn table_row(id: &str, name: &str, triangles: i64) -> UiValue {
    let mut row = BTreeMap::new();
    row.insert("id".to_string(), UiValue::String(id.to_string()));
    row.insert("name".to_string(), UiValue::String(name.to_string()));
    row.insert("triangles".to_string(), UiValue::Int(triangles));
    UiValue::Map(row)
}

fn table_column(field: &str, width: f64) -> UiValue {
    let mut column = BTreeMap::new();
    column.insert("field".to_string(), UiValue::String(field.to_string()));
    column.insert("width".to_string(), UiValue::Float(width));
    UiValue::Map(column)
}

fn column_width_payload(field: &str, width: f64) -> UiValue {
    let mut payload = BTreeMap::new();
    payload.insert("field".to_string(), UiValue::String(field.to_string()));
    payload.insert("width".to_string(), UiValue::Float(width));
    UiValue::Map(payload)
}

fn sort_model(field: &str, direction: &str) -> UiValue {
    let mut entry = BTreeMap::new();
    entry.insert("field".to_string(), UiValue::String(field.to_string()));
    entry.insert("sort".to_string(), UiValue::String(direction.to_string()));
    UiValue::Array(vec![UiValue::Map(entry)])
}

fn assert_row_ids<const N: usize>(state: &UiComponentState, expected: [&str; N]) {
    let rows = match state.value("rows") {
        Some(UiValue::Array(rows)) => rows,
        other => panic!("expected rows array, got {other:?}"),
    };
    let actual: Vec<_> = rows
        .iter()
        .map(|row| match row {
            UiValue::Map(row) => match row.get("id") {
                Some(UiValue::String(id)) => id.as_str(),
                other => panic!("expected row id string, got {other:?}"),
            },
            other => panic!("expected row map, got {other:?}"),
        })
        .collect();
    assert_eq!(actual, expected);
}

fn assert_column_width(state: &UiComponentState, field: &str, expected: f64) {
    let columns = match state.value("columns") {
        Some(UiValue::Array(columns)) => columns,
        other => panic!("expected columns array, got {other:?}"),
    };
    for column in columns {
        let UiValue::Map(column) = column else {
            continue;
        };
        if matches!(column.get("field"), Some(UiValue::String(value)) if value == field) {
            assert_eq!(column.get("width"), Some(&UiValue::Float(expected)));
            return;
        }
    }
    panic!("expected width for column `{field}`");
}
