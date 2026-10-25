//! In-memory source candidates only; no registered project, schema, provider or document fixture.

use serde_json::json;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectTypeInfo, ReflectedValue};

use super::super::{
    DataTableCellCodec, DataTableCellError, DataTableSourceAdapter, DataTableSourceError,
};
use super::{asset, field};

fn remove(
    source: &DataAsset,
    columns: Vec<ReflectFieldInfo>,
    keys: &[&str],
) -> Result<Option<DataAsset>, DataTableCellError> {
    DataTableCellCodec::prepare_rows_remove(
        source,
        Some(&ReflectTypeInfo::struct_with_fields(columns)),
        "Name",
        keys,
    )
}

fn duplicate(
    source: &DataAsset,
    columns: Vec<ReflectFieldInfo>,
    key: &str,
    new_key: &str,
) -> Result<DataAsset, DataTableCellError> {
    DataTableCellCodec::prepare_row_duplicate(
        source,
        Some(&ReflectTypeInfo::struct_with_fields(columns)),
        "Name",
        key,
        new_key,
    )
}

#[test]
fn data_table_row_ops_remove_first_middle_last_and_all_preserves_outer_bytes() {
    let source = asset(" \n[\n {\"Name\":\"a\",\"v\":1}, \t{\"Name\":\"b\",\"v\":2},\r\n {\"Name\":\"c\",\"v\":3}\n]\t");
    let original = source.clone();
    for (keys, expected) in [
        (
            vec!["a"],
            " \n[\n {\"Name\":\"b\",\"v\":2},\r\n {\"Name\":\"c\",\"v\":3}\n]\t",
        ),
        (
            vec!["b"],
            " \n[\n {\"Name\":\"a\",\"v\":1}, \t{\"Name\":\"c\",\"v\":3}\n]\t",
        ),
        (
            vec!["c"],
            " \n[\n {\"Name\":\"a\",\"v\":1}, \t{\"Name\":\"b\",\"v\":2}\n]\t",
        ),
        (vec!["c", "a", "b"], " \n[\n \n]\t"),
    ] {
        let candidate = remove(&source, vec![field("v", "u64")], &keys)
            .unwrap()
            .unwrap();
        assert_eq!(candidate.text, expected);
        assert_eq!(candidate.uri, source.uri);
        assert_eq!(candidate.format, source.format);
    }
    assert_eq!(source, original);
}

#[test]
fn data_table_row_ops_remove_every_subset_by_key_keeps_source_order_and_survivor_tokens() {
    let raw = [
        r#"{"Name":"z","v":18446744073709551615,"f":-0.000e+9}"#,
        r#"{"Name":"a","v":9007199254740993,"f":1.2500}"#,
        r#"{"Name":"m","v":0,"f":-2.5000}"#,
        r#"{"Name":"末","v":7,"f":0.000}"#,
    ];
    let source = asset(&format!("[{}]", raw.join(",")));
    let row_type = ReflectTypeInfo::struct_with_fields(vec![field("v", "u64"), field("f", "f32")]);
    let keys = ["z", "a", "m", "末"];
    for mask in 1..16 {
        let selected: Vec<_> = keys
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(index, key)| (mask & (1 << index) != 0).then_some(*key))
            .collect();
        let candidate =
            DataTableCellCodec::prepare_rows_remove(&source, Some(&row_type), "Name", &selected)
                .unwrap()
                .unwrap();
        let survivors: Vec<_> = raw
            .iter()
            .enumerate()
            .filter_map(|(index, row)| (mask & (1 << index) == 0).then_some(*row))
            .collect();
        assert_eq!(candidate.text, format!("[{}]", survivors.join(",")));
        let reopened = DataTableSourceAdapter::read(&candidate, Some(&row_type), "Name").unwrap();
        if mask & 1 == 0 {
            assert_eq!(
                reopened.rows[0].cells[0].value,
                ReflectedValue::Unsigned(u64::MAX)
            );
            let ReflectedValue::Scalar(value) = &reopened.rows[0].cells[1].value else {
                panic!("f32 cell expected")
            };
            assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
        }
    }
}

#[test]
fn data_table_row_ops_remove_rejects_duplicate_or_missing_keys_without_partial_candidate() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"b","v":2}]"#);
    let original = source.clone();
    let columns = vec![field("v", "u64")];
    assert_eq!(
        remove(&source, columns.clone(), &["a", "a"]),
        Err(DataTableCellError::DuplicateRowSelection("a".into()))
    );
    assert_eq!(
        remove(&source, columns, &["a", "missing"]),
        Err(DataTableCellError::MissingRow("missing".into()))
    );
    assert_eq!(source, original);
}

#[test]
fn data_table_row_ops_remove_noop_and_mutation_preflight_unselected_source_and_cache() {
    let columns = vec![field("v", "u64")];
    let source = asset(" \n[]\t");
    assert_eq!(remove(&source, columns.clone(), &[]), Ok(None));
    let invalid = asset(r#"[{"Name":"a","v":1},{"Name":"bad","v":"wrong"}]"#);
    for keys in [vec![], vec!["a"]] {
        assert!(matches!(
            remove(&invalid, columns.clone(), &keys),
            Err(DataTableCellError::Source(
                DataTableSourceError::InvalidCell { .. }
            ))
        ));
    }
    let mut stale = asset(r#"[{"Name":"a","v":1}]"#);
    stale.canonical_json = json!([{"Name":"a","v":2}]);
    assert_eq!(
        remove(&stale, columns, &[]),
        Err(DataTableSourceError::CanonicalMismatch.into())
    );
    assert_eq!(
        DataTableCellCodec::prepare_rows_remove(&source, None, "Name", &[]),
        Err(DataTableSourceError::MissingRowType.into())
    );
}

#[test]
fn data_table_row_ops_duplicate_copies_full_raw_row_and_appends_in_source_order() {
    let source = asset(" \n[\n {\"Na\\u006de\":\"\\u65e7行\",\"label\":\"旧行\",\"n\":18446744073709551615,\"f\":-0.000e+9},\n {\"Name\":\"other\",\"label\":\"a\",\"n\":0,\"f\":1.2500}\n]\t");
    let original = source.clone();
    let columns = vec![
        field("label", "alloc::string::String").with_editable(false),
        field("n", "u64"),
        field("f", "f32"),
    ];
    let candidate = duplicate(&source, columns.clone(), "旧行", "新\"行\n🧩").unwrap();
    let copied = "{\"Na\\u006de\":\"新\\\"行\\n🧩\",\"label\":\"旧行\",\"n\":18446744073709551615,\"f\":-0.000e+9}";
    let expected = source
        .text
        .replacen("\n]\t", &format!(",\n {copied}\n]\t"), 1);
    assert_eq!(candidate.text, expected);
    assert_eq!(candidate.uri, source.uri);
    assert_eq!(candidate.format, source.format);
    assert_eq!(source, original);
    let row_type = ReflectTypeInfo::struct_with_fields(columns);
    let reopened = DataTableSourceAdapter::read(&candidate, Some(&row_type), "Name").unwrap();
    assert_eq!(
        reopened
            .rows
            .iter()
            .map(|row| row.source_key.as_str())
            .collect::<Vec<_>>(),
        vec!["旧行", "other", "新\"行\n🧩"]
    );
    assert_eq!(reopened.rows[2].cells, reopened.rows[0].cells);
    let ReflectedValue::Scalar(value) = &reopened.rows[2].cells[2].value else {
        panic!("f32 cell expected")
    };
    assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
}

#[test]
fn data_table_row_ops_duplicate_rejects_collision_empty_target_and_missing_source() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"A","v":2}]"#);
    let columns = vec![field("v", "u64")];
    for new_key in ["a", "A"] {
        assert_eq!(
            duplicate(&source, columns.clone(), "a", new_key),
            Err(DataTableSourceError::DuplicateRowKey(new_key.into()).into())
        );
    }
    assert_eq!(
        duplicate(&source, columns.clone(), "a", ""),
        Err(DataTableSourceError::InvalidRowKey {
            source_row: 2,
            key_field: "Name".into()
        }
        .into())
    );
    assert_eq!(
        duplicate(&source, columns.clone(), "missing", "new"),
        Err(DataTableCellError::MissingRow("missing".into()))
    );
    assert!(duplicate(&source, columns, "a", "None").is_ok());
}

#[test]
fn data_table_row_ops_duplicate_preflights_whole_source_schema_format_and_cache() {
    let columns = vec![field("v", "u64")];
    let invalid = asset(r#"[{"Name":"a","v":1},{"Name":"bad","v":"wrong"}]"#);
    assert!(matches!(
        duplicate(&invalid, columns.clone(), "a", "new"),
        Err(DataTableCellError::Source(
            DataTableSourceError::InvalidCell { .. }
        ))
    ));
    let mut source = asset(r#"[{"Name":"a","v":1}]"#);
    assert_eq!(
        DataTableCellCodec::prepare_row_duplicate(&source, None, "Name", "a", "new"),
        Err(DataTableSourceError::MissingRowType.into())
    );
    source.format = DataAssetFormat::Toml;
    assert_eq!(
        duplicate(&source, columns.clone(), "a", "new"),
        Err(DataTableSourceError::UnsupportedFormat(DataAssetFormat::Toml).into())
    );
    source.format = DataAssetFormat::Json;
    source.canonical_json = json!([{"Name":"a","v":2}]);
    assert_eq!(
        duplicate(&source, columns, "a", "new"),
        Err(DataTableSourceError::CanonicalMismatch.into())
    );
}

#[test]
fn data_table_row_ops_custom_escaped_key_and_unicode_identity_are_exact() {
    let source = asset(r#"[{"\u0069d":"é","v":1},{"id":"é","v":2},{"id":"A","v":3}]"#);
    let row_type = ReflectTypeInfo::struct_with_fields(vec![field("v", "u64")]);
    let candidate = DataTableCellCodec::prepare_rows_remove(&source, Some(&row_type), "id", &["é"])
        .unwrap()
        .unwrap();
    assert_eq!(candidate.text, r#"[{"id":"é","v":2},{"id":"A","v":3}]"#);
    let duplicate =
        DataTableCellCodec::prepare_row_duplicate(&source, Some(&row_type), "id", "A", "a")
            .unwrap();
    assert_eq!(duplicate.canonical_json[3], json!({"id":"a","v":3}));
    assert_eq!(source.canonical_json, serde_json::Value::Null);
}

#[test]
fn data_table_row_ops_single_key_only_row_preserves_brackets_and_trailing_whitespace() {
    let source = asset(" \t[\r\n {\"Name\":\"a\"}\r\n]\n");
    let copied = duplicate(&source, vec![], "a", "b").unwrap();
    assert_eq!(
        copied.text,
        " \t[\r\n {\"Name\":\"a\"},{\"Name\":\"b\"}\r\n]\n"
    );
    let removed = remove(&source, vec![], &["a"]).unwrap().unwrap();
    assert_eq!(removed.text, " \t[\r\n \r\n]\n");
    assert_eq!(removed.canonical_json, json!([]));
}
