//! Pure candidate preparation, without schema registration, references or document admission.

use serde_json::json;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectTypeInfo, ReflectedValue};

use super::super::{
    DataTableCellCodec, DataTableCellError, DataTableSourceAdapter, DataTableSourceError,
};
use super::{asset, field};

fn rename(
    source: &DataAsset,
    columns: Vec<ReflectFieldInfo>,
    old_key: &str,
    new_key: &str,
) -> Result<Option<DataAsset>, DataTableCellError> {
    DataTableCellCodec::prepare_row_rename(
        source,
        Some(&ReflectTypeInfo::struct_with_fields(columns)),
        "Name",
        old_key,
        new_key,
    )
}

#[test]
fn data_table_row_rename_preserves_all_other_bytes_and_immutable_source() {
    let source = asset("[\n {\"Na\\u006de\" : \"\\u65e7行\", \"label\":\"旧行\", \"n\":18446744073709551615, \"f\":-0.000e+9},\n {\"Name\":\"other\",\"label\":\"\\u65e7行\",\"n\":0,\"f\":1.2500}\n]\t");
    let original = source.clone();
    let columns = vec![
        field("label", "alloc::string::String"),
        field("n", "u64"),
        field("f", "f32"),
    ];
    let candidate = rename(&source, columns.clone(), "旧行", "新\"行\n🧩")
        .unwrap()
        .unwrap();
    let expected = source
        .text
        .replacen("\"\\u65e7行\"", "\"新\\\"行\\n🧩\"", 1);
    assert_eq!(candidate.text, expected);
    assert_eq!(candidate.uri, source.uri);
    assert_eq!(candidate.format, source.format);
    assert_eq!(source, original);
    let row_type = ReflectTypeInfo::struct_with_fields(columns);
    let before = DataTableSourceAdapter::read(&source, Some(&row_type), "Name").unwrap();
    let after = DataTableSourceAdapter::read(&candidate, Some(&row_type), "Name").unwrap();
    assert_eq!(after.rows[0].source_key, "新\"行\n🧩");
    assert_eq!(after.rows[1], before.rows[1]);
    assert_eq!(
        after.rows[0].cells[1].value,
        ReflectedValue::Unsigned(u64::MAX)
    );
    let ReflectedValue::Scalar(value) = &after.rows[0].cells[2].value else {
        panic!("f32 cell expected")
    };
    assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
}

#[test]
fn data_table_row_rename_uses_exact_key_and_preserves_source_row_order() {
    let source = asset(r#"[{"Name":"z","v":1},{"Name":"a","v":2},{"Name":"m","v":3}]"#);
    let candidate = rename(&source, vec![field("v", "u64")], "a", "é")
        .unwrap()
        .unwrap();
    assert_eq!(
        candidate.text,
        r#"[{"Name":"z","v":1},{"Name":"é","v":2},{"Name":"m","v":3}]"#
    );
    assert_eq!(candidate.canonical_json[1]["Name"], "é");
}

#[test]
fn data_table_row_rename_rejects_collisions_missing_rows_and_empty_targets() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"b","v":2}]"#);
    let original = source.clone();
    let columns = vec![field("v", "u64")];
    assert_eq!(
        rename(&source, columns.clone(), "a", "b"),
        Err(DataTableCellError::Source(
            DataTableSourceError::DuplicateRowKey("b".into())
        ))
    );
    assert_eq!(
        rename(&source, columns.clone(), "missing", "missing"),
        Err(DataTableCellError::MissingRow("missing".into()))
    );
    assert_eq!(
        rename(&source, columns, "b", ""),
        Err(DataTableCellError::Source(
            DataTableSourceError::InvalidRowKey {
                source_row: 1,
                key_field: "Name".into()
            }
        ))
    );
    assert_eq!(source, original);
}

#[test]
fn data_table_row_rename_noop_validates_whole_source_without_reformatting() {
    let source = asset(" \n[{\"Name\":\"a\",\"v\":1.2500}]\t");
    assert_eq!(rename(&source, vec![field("v", "f32")], "a", "a"), Ok(None));
    assert!(source.canonical_json.is_null());
    let invalid = asset(r#"[{"Name":"a","v":1},{"Name":"bad","v":"wrong"}]"#);
    assert!(matches!(
        rename(&invalid, vec![field("v", "u64")], "a", "a"),
        Err(DataTableCellError::Source(
            DataTableSourceError::InvalidCell { .. }
        ))
    ));
}

#[test]
fn data_table_row_rename_rejects_missing_schema_unsupported_format_and_stale_cache() {
    let mut source = asset(r#"[{"Name":"a","v":1}]"#);
    assert_eq!(
        DataTableCellCodec::prepare_row_rename(&source, None, "Name", "a", "a"),
        Err(DataTableCellError::Source(
            DataTableSourceError::MissingRowType
        ))
    );
    source.format = DataAssetFormat::Toml;
    assert_eq!(
        rename(&source, vec![field("v", "u64")], "a", "new"),
        Err(DataTableCellError::Source(
            DataTableSourceError::UnsupportedFormat(DataAssetFormat::Toml)
        ))
    );
    source.format = DataAssetFormat::Json;
    source.canonical_json = json!([{"Name":"a","v":2}]);
    assert_eq!(
        rename(&source, vec![field("v", "u64")], "a", "a"),
        Err(DataTableCellError::Source(
            DataTableSourceError::CanonicalMismatch
        ))
    );
}

#[test]
fn data_table_row_rename_follows_json_keys_without_fname_case_or_unicode_aliases() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"A","v":2},{"Name":"é","v":3}]"#);
    let columns = vec![field("v", "u64")];
    assert!(rename(&source, columns.clone(), "a", "e\u{301}")
        .unwrap()
        .is_some());
    assert_eq!(
        rename(&source, columns.clone(), "a", "A"),
        Err(DataTableCellError::Source(
            DataTableSourceError::DuplicateRowKey("A".into())
        ))
    );
    let candidate = rename(&source, columns, "a", "None").unwrap().unwrap();
    assert_eq!(candidate.canonical_json[0]["Name"], "None");
}

#[test]
fn data_table_row_rename_resolves_escaped_custom_key_field_without_column_mutation() {
    let source = asset(r#"[{"\u0069d":"old","value":-0.0},{"id":"newer","value":5}]"#);
    let mut value = field("value", "f32");
    value.editable = false;
    value.editor_visible = false;
    let row_type = ReflectTypeInfo::struct_with_fields(vec![value]);
    let candidate =
        DataTableCellCodec::prepare_row_rename(&source, Some(&row_type), "id", "old", "new")
            .unwrap()
            .unwrap();
    assert_eq!(
        candidate.text,
        r#"[{"\u0069d":"new","value":-0.0},{"id":"newer","value":5}]"#
    );
    assert_eq!(
        DataTableCellCodec::prepare_row_rename(&source, Some(&row_type), "Name", "old", "new"),
        Err(DataTableCellError::Source(
            DataTableSourceError::InvalidRowKey {
                source_row: 0,
                key_field: "Name".into()
            }
        ))
    );
}
