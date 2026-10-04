use super::*;

#[test]
fn optimization_batch_20260909_runtime438_same_width_move_is_a_noop() {
    assert!(!table_column_width_changed(96.0, 96.0));
    assert!(table_column_width_changed(96.0, 96.5));
}

#[test]
fn optimization_batch_20260909_runtime438_stale_column_projection_is_not_a_noop() {
    let metadata = UiTemplateNodeMetadata {
        attributes: std::collections::BTreeMap::from([
            (
                "column_widths".to_string(),
                toml::Value::Table(toml::map::Map::from_iter([(
                    "name".to_string(),
                    toml::Value::Float(96.0),
                )])),
            ),
            (
                "columns".to_string(),
                toml::Value::Array(vec![toml::Value::Table(toml::map::Map::from_iter([
                    ("field".to_string(), toml::Value::String("name".to_string())),
                    ("width".to_string(), toml::Value::Float(72.0)),
                ]))]),
            ),
        ]),
        ..UiTemplateNodeMetadata::default()
    };
    assert!(!columns::table_column_width_projection_is_current(
        &metadata, "name", 96.0
    ));
}

#[test]
fn optimization_batch_20260909_runtime438_resize_path_checks_noop_before_projection() {
    let source = include_str!("../resize.rs");
    let start = source
        .find("fn apply_default_table_column_width(")
        .expect("table width route");
    let body = &source[start..];
    assert!(
        body.find("table_column_width_changed(previous_width, width)")
            .expect("same-width guard")
            < body
                .find("apply_table_column_width_batch(")
                .expect("batched projection")
    );
    let drag_start = source
        .find("fn apply_default_table_column_resize_drag(")
        .expect("drag route");
    let release_start = source
        .find("fn apply_default_table_column_resize_release(")
        .expect("release route");
    let drag_body = &source[drag_start..release_start];
    assert!(drag_body.contains("pointer_drag_resize(owner_id)"));
}
