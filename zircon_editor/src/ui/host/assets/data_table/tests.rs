//! Pure in-memory inputs, never registered project assets or a schema/provider fixture.

use serde_json::json;
use zircon_runtime::asset::{AssetUri, DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectTypeInfo, ReflectedValue,
};

use super::*;

fn field(name: &str, path: &str) -> ReflectFieldInfo {
    ReflectFieldInfo::from_stable_keys(
        "test.data_table.row",
        name,
        name,
        path,
        ReflectEditorHint::None,
    )
}

fn asset(text: &str) -> DataAsset {
    DataAsset {
        uri: AssetUri::parse("res://unit-memory-only.json").unwrap(),
        format: DataAssetFormat::Json,
        text: text.into(),
        canonical_json: serde_json::Value::Null,
    }
}

fn read(
    text: &str,
    fields: Vec<ReflectFieldInfo>,
) -> Result<DataTableSource, DataTableSourceError> {
    DataTableSourceAdapter::read(
        &asset(text),
        Some(&ReflectTypeInfo::struct_with_fields(fields)),
        "Name",
    )
}

#[test]
fn data_table_requires_explicit_row_type_and_json_format() {
    let mut source = asset("[]");
    assert_eq!(
        DataTableSourceAdapter::read(&source, None, "Name"),
        Err(DataTableSourceError::MissingRowType)
    );
    source.format = DataAssetFormat::Toml;
    assert!(matches!(
        DataTableSourceAdapter::read(&source, None, "Name"),
        Err(DataTableSourceError::UnsupportedFormat(
            DataAssetFormat::Toml
        ))
    ));
}

#[test]
fn data_table_preserves_integer_limits_ids_metadata_and_source_order() {
    let count = field("Count", "u64").with_editable(false);
    let delta = field("Delta", "i64");
    let table = read(
        r#"[{"Name":"行二","Delta":-9223372036854775808,"Count":18446744073709551615},{"Name":"行一","Count":9007199254740993,"Delta":9223372036854775807}]"#,
        vec![count.clone(), delta],
    )
    .unwrap();
    assert_eq!(table.columns[0], count);
    assert_eq!(table.rows[0].source_key, "行二");
    assert_eq!(table.rows[1].source_key, "行一");
    assert_eq!(table.rows[0].cells[0].field_id, count.id);
    assert_eq!(
        table.rows[0].cells[0].value,
        ReflectedValue::Unsigned(u64::MAX)
    );
    assert_eq!(
        table.rows[0].cells[1].value,
        ReflectedValue::Integer(i64::MIN)
    );
    assert_eq!(
        table.rows[1].cells[0].value,
        ReflectedValue::Unsigned(9007199254740993)
    );
    assert_eq!(
        table.rows[1].cells[1].value,
        ReflectedValue::Integer(i64::MAX)
    );
}

#[test]
fn data_table_checks_declared_integer_width_without_numeric_coercion() {
    for (path, valid, invalid) in [
        ("i8", "-128", "128"),
        ("i16", "32767", "32768"),
        ("i32", "-2147483648", "2147483648"),
        ("i64", "9223372036854775807", "9223372036854775808"),
        ("u8", "255", "256"),
        ("u16", "65535", "65536"),
        ("u32", "4294967295", "4294967296"),
        ("u64", "18446744073709551615", "18446744073709551616"),
    ] {
        assert!(read(
            &format!(r#"[{{"Name":"a","v":{valid}}}]"#),
            vec![field("v", path)]
        )
        .is_ok());
        for token in [invalid, "1.0", "1e0", "null", "\"1\"", "true"] {
            assert!(
                matches!(
                    read(
                        &format!(r#"[{{"Name":"a","v":{token}}}]"#),
                        vec![field("v", path)]
                    ),
                    Err(DataTableSourceError::InvalidCell { .. })
                ),
                "{path}: {token}"
            );
        }
    }
    assert!(read(r#"[{"Name":"a","v":-1}]"#, vec![field("v", "u64")]).is_err());
}

#[test]
fn data_table_reads_bool_and_unicode_text_without_string_coercion() {
    let table = read(
        r#"[{"Name":"路径/原键","enabled":true,"text":"中文与长文本\n\"保留\""}]"#,
        vec![
            field("enabled", "bool"),
            field("text", "alloc::string::String"),
        ],
    )
    .unwrap();
    assert_eq!(table.rows[0].cells[0].value, ReflectedValue::Bool(true));
    assert_eq!(
        table.rows[0].cells[1].value,
        ReflectedValue::String("中文与长文本\n\"保留\"".into())
    );
    assert!(read(r#"[{"Name":"a","v":1}]"#, vec![field("v", "bool")]).is_err());
    assert!(read(
        r#"[{"Name":"a","v":true}]"#,
        vec![field("v", "alloc::string::String")]
    )
    .is_err());
}

#[test]
fn data_table_rejects_float_precision_loss_before_f64_rounding() {
    for token in ["0.1", "1e0", "1.2500", "-0.0", "16777216"] {
        assert!(
            read(
                &format!(r#"[{{"Name":"a","v":{token}}}]"#),
                vec![field("v", "f32")]
            )
            .is_ok(),
            "{token}"
        );
    }
    for token in [
        "1.000000000000000000000001",
        "16777217",
        "1e-100",
        "1e100",
        "null",
        "\"1\"",
    ] {
        assert!(
            matches!(
                read(
                    &format!(r#"[{{"Name":"a","v":{token}}}]"#),
                    vec![field("v", "f32")]
                ),
                Err(DataTableSourceError::InvalidCell { .. })
            ),
            "{token}"
        );
    }
}

#[test]
fn data_table_rejects_unsupported_columns_even_when_empty() {
    for path in [
        "f64",
        "String",
        "game.Resource",
        "Vec3",
        "serde_json::Value",
        "Vec<u32>",
    ] {
        assert!(matches!(
            read("[]", vec![field("v", path)]),
            Err(DataTableSourceError::UnsupportedColumn { .. })
        ));
    }
}

#[test]
fn data_table_rejects_duplicate_json_properties_before_map_collapse() {
    for text in [
        r#"[{"Name":"a","Name":"b","v":1}]"#,
        r#"[{"Name":"a","v":1,"v":2}]"#,
    ] {
        assert!(matches!(
            read(text, vec![field("v", "u32")]),
            Err(DataTableSourceError::Encoding(_))
        ));
    }
}

#[test]
fn data_table_requires_original_nonempty_unique_source_keys() {
    for text in [
        r#"[{"v":1}]"#,
        r#"[{"Name":"","v":1}]"#,
        r#"[{"Name":1,"v":1}]"#,
    ] {
        assert!(matches!(
            read(text, vec![field("v", "u32")]),
            Err(DataTableSourceError::InvalidRowKey { .. })
        ));
    }
    assert_eq!(
        read(
            r#"[{"Name":"a","v":1},{"Name":"a","v":2}]"#,
            vec![field("v", "u32")]
        ),
        Err(DataTableSourceError::DuplicateRowKey("a".into()))
    );
    let table = read(
        r#"[{"Name":" A ","v":1},{"Name":"a","v":2}]"#,
        vec![field("v", "u32")],
    )
    .unwrap();
    assert_eq!(table.rows[0].source_key, " A ");
    assert_eq!(table.rows[1].source_key, "a");
}

#[test]
fn data_table_rejects_unknown_missing_alias_and_nonserializable_columns() {
    let column = field("v", "u32").with_aliases(vec!["old".into()]);
    for text in [
        r#"[{"Name":"a","v":1,"extra":2}]"#,
        r#"[{"Name":"a","old":1}]"#,
    ] {
        assert!(matches!(
            read(text, vec![column.clone()]),
            Err(DataTableSourceError::UnknownColumn { .. })
        ));
    }
    assert!(matches!(
        read(r#"[{"Name":"a"}]"#, vec![column]),
        Err(DataTableSourceError::MissingColumn { .. })
    ));
    assert!(matches!(
        read(
            r#"[{"Name":"a","v":1}]"#,
            vec![field("v", "u32").with_serializable(false)]
        ),
        Err(DataTableSourceError::UnknownColumn { .. })
    ));
}

#[test]
fn data_table_rejects_ambiguous_row_metadata_and_key_collision() {
    let column = field("v", "u32");
    let mut other = field("other", "u32");
    other.id = column.id;
    for fields in [
        vec![column.clone(), column],
        vec![field("v", "u32"), other],
        vec![field("Name", "u32")],
    ] {
        assert!(matches!(
            read("[]", fields),
            Err(DataTableSourceError::InvalidRowType(_))
        ));
    }
    assert!(matches!(
        DataTableSourceAdapter::read(&asset("[]"), Some(&ReflectTypeInfo::opaque()), "Name"),
        Err(DataTableSourceError::InvalidRowType(_))
    ));
    assert!(matches!(
        DataTableSourceAdapter::read(
            &asset("[]"),
            Some(&ReflectTypeInfo::struct_with_fields(vec![])),
            ""
        ),
        Err(DataTableSourceError::InvalidRowType(_))
    ));
}

#[test]
fn data_table_preserves_source_and_rejects_stale_canonical_cache() {
    let mut source = asset(" [ { \"Name\":\"a\", \"v\":42 } ] \n");
    let row_type = ReflectTypeInfo::struct_with_fields(vec![field("v", "u32")]);
    let before = source.clone();
    DataTableSourceAdapter::read(&source, Some(&row_type), "Name").unwrap();
    assert_eq!(source, before);
    source.canonical_json = json!([{"Name":"a","v":43}]);
    let stale = source.clone();
    assert_eq!(
        DataTableSourceAdapter::read(&source, Some(&row_type), "Name"),
        Err(DataTableSourceError::CanonicalMismatch)
    );
    assert_eq!(source, stale);
}

#[test]
fn data_table_supports_explicit_key_field_empty_rows_and_rejects_non_table_json() {
    let row_type = ReflectTypeInfo::struct_with_fields(vec![field("v", "u32")]);
    let table = DataTableSourceAdapter::read(
        &asset(r#"[{"行键":"原始","v":2}]"#),
        Some(&row_type),
        "行键",
    )
    .unwrap();
    assert_eq!(table.rows[0].source_key, "原始");
    assert_eq!(
        table.rows[0].cells[0].field_id,
        ReflectFieldId::from_stable_keys("test.data_table.row", "v")
    );
    assert!(
        DataTableSourceAdapter::read(&asset("[]"), Some(&row_type), "Name")
            .unwrap()
            .rows
            .is_empty()
    );
    for text in ["{}", "null", "[1]", "[null]", "[{}]", "[] trailing"] {
        assert!(DataTableSourceAdapter::read(&asset(text), Some(&row_type), "Name").is_err());
    }
}

#[path = "tests/editing.rs"]
mod editing;

#[path = "tests/row_rename.rs"]
mod row_rename;

#[path = "tests/row_ops.rs"]
mod row_ops;

#[path = "tests/csv.rs"]
mod csv;

#[path = "tests/row_move.rs"]
mod row_move;
