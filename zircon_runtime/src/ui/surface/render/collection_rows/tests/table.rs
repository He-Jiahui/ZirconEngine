use super::*;

#[test]
fn table_cells_materialize_only_the_four_rendered_columns() {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata.attributes.insert(
        "cells".to_string(),
        Value::Array(
            (0..128)
                .map(|index| Value::String(format!("cell-{index}")))
                .collect(),
        ),
    );

    assert_eq!(
        table_cells(&metadata),
        [
            Some("cell-0".to_string()),
            Some("cell-1".to_string()),
            Some("cell-2".to_string()),
            Some("cell-3".to_string()),
        ]
    );
}

#[test]
fn empty_cell_array_preserves_compact_row_label_fallback() {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata.attributes.insert(
        "cells".to_string(),
        Value::Array(vec![Value::String("  ".to_string())]),
    );
    metadata.attributes.insert(
        "label".to_string(),
        Value::String("texture image 12 KiB today UTC".to_string()),
    );

    assert_eq!(
        table_cells(&metadata),
        [
            Some("texture".to_string()),
            Some("image".to_string()),
            Some("12 KiB".to_string()),
            Some("today UTC".to_string()),
        ]
    );
}
