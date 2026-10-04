use std::collections::BTreeMap;

use super::*;

#[test]
fn component_matching_accepts_the_authored_role_alias() {
    let metadata = UiTemplateNodeMetadata {
        component: "Panel".into(),
        attributes: BTreeMap::from([(
            "component_role".into(),
            Value::String("mui-x-data-grid".into()),
        )]),
        ..UiTemplateNodeMetadata::default()
    };
    assert!(component_matches(
        &metadata,
        &["DataGrid", "mui-x-data-grid"]
    ));
}

#[test]
fn string_array_filters_nonsemantic_values() {
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([(
            "items".into(),
            Value::Array(vec![
                Value::String("first".into()),
                Value::String("  ".into()),
            ]),
        )]),
        ..UiTemplateNodeMetadata::default()
    };
    assert_eq!(string_array(&metadata, "items"), vec!["first"]);
}

#[test]
fn collection_window_clamps_offset_and_physical_capacity() {
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("viewport_start".into(), Value::Integer(2)),
            ("visible_limit".into(), Value::Integer(20)),
            (
                "items".into(),
                Value::Array(
                    (0..8)
                        .map(|index| Value::String(format!("item-{index}")))
                        .collect(),
                ),
            ),
        ]),
        ..UiTemplateNodeMetadata::default()
    };

    assert_eq!(
        collection_window(&metadata, "items", 3),
        vec!["item-2", "item-3", "item-4"]
    );
}

#[test]
fn collection_window_returns_no_rows_when_the_parent_has_no_capacity() {
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([(
            "items".into(),
            Value::Array(vec![Value::String("item".into())]),
        )]),
        ..UiTemplateNodeMetadata::default()
    };

    assert!(collection_window(&metadata, "items", 0).is_empty());
}

#[test]
fn collection_window_clamps_negative_and_past_end_offsets() {
    let values = (0..4)
        .map(|index| Value::String(format!("item-{index}")))
        .collect::<Vec<_>>();
    let negative_start = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("viewport_start".into(), Value::Integer(-5)),
            ("visible_limit".into(), Value::Integer(2)),
            ("items".into(), Value::Array(values.clone())),
        ]),
        ..UiTemplateNodeMetadata::default()
    };
    let past_end = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("viewport_start".into(), Value::Integer(99)),
            ("visible_limit".into(), Value::Integer(2)),
            ("items".into(), Value::Array(values)),
        ]),
        ..UiTemplateNodeMetadata::default()
    };

    assert_eq!(
        collection_window(&negative_start, "items", 2),
        vec!["item-0", "item-1"]
    );
    assert!(collection_window(&past_end, "items", 2).is_empty());
}

#[test]
fn collection_window_honors_an_explicit_zero_limit() {
    let metadata = UiTemplateNodeMetadata {
        attributes: BTreeMap::from([
            ("visible_limit".into(), Value::Integer(0)),
            (
                "items".into(),
                Value::Array(vec![Value::String("item".into())]),
            ),
        ]),
        ..UiTemplateNodeMetadata::default()
    };

    assert!(collection_window(&metadata, "items", 2).is_empty());
}
