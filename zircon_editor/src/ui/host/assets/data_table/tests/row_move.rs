//! Pure in-memory inputs, never registered assets, live table documents or a provider fixture.

use serde_json::json;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectTypeInfo, ReflectedValue};

use super::super::{
    DataTableCellCodec, DataTableCellError, DataTableSourceAdapter, DataTableSourceError,
};
use super::{asset, field};

fn move_row(
    source: &DataAsset,
    fields: Vec<ReflectFieldInfo>,
    key: &str,
    target: &str,
) -> Result<Option<DataAsset>, DataTableCellError> {
    DataTableCellCodec::prepare_row_move(
        source,
        Some(&ReflectTypeInfo::struct_with_fields(fields)),
        "Name",
        key,
        target,
    )
}

#[test]
fn data_table_row_move_uses_exact_keys_and_original_target_source_position() {
    // Deliberately nonalphabetic order: a display sort must not become source identity.
    let source =
        asset(r#"[{"Name":"z","v":1},{"Name":"a","v":2},{"Name":"m","v":3},{"Name":"b","v":4}]"#);
    let original = source.clone();
    let row_type = ReflectTypeInfo::struct_with_fields(vec![field("v", "u64")]);
    for (key, target, expected) in [
        ("z", "a", ["a", "z", "m", "b"]),
        ("z", "m", ["a", "m", "z", "b"]),
        ("a", "b", ["z", "m", "b", "a"]),
        ("m", "z", ["m", "z", "a", "b"]),
        ("b", "a", ["z", "b", "a", "m"]),
        ("b", "z", ["b", "z", "a", "m"]),
    ] {
        let candidate =
            DataTableCellCodec::prepare_row_move(&source, Some(&row_type), "Name", key, target)
                .unwrap()
                .unwrap();
        let reopened = DataTableSourceAdapter::read(&candidate, Some(&row_type), "Name").unwrap();
        let keys: Vec<_> = reopened
            .rows
            .iter()
            .map(|row| row.source_key.as_str())
            .collect();
        assert_eq!(keys, expected);
        assert_eq!(candidate.uri, original.uri);
        assert_eq!(candidate.format, original.format);
        assert_eq!(candidate.text.len(), original.text.len());
    }
    assert_eq!(source, original);
}

#[test]
fn data_table_row_move_preserves_full_row_literals_and_positional_separators() {
    let rows = [
        r#"{"Na\u006de":"\u7532","n":18446744073709551615,"f":-0.000e+9,"s":"中,文\n\"引号\""}"#,
        r#"{ "Name":"乙", "n":9007199254740993, "f":1.2500, "s":"相同文本" }"#,
        r#"{"s":"相同文本","f":0.000,"n":0,"Name":"🧩"}"#,
    ];
    let source = asset(&format!(
        " \r\n[\n {} ,\t{}\r\n,  {} \n]\t",
        rows[0], rows[1], rows[2]
    ));
    let original = source.clone();
    let fields = vec![
        field("n", "u64"),
        field("f", "f32"),
        field("s", "alloc::string::String"),
    ];
    let candidate = move_row(&source, fields.clone(), "甲", "🧩")
        .unwrap()
        .unwrap();
    assert_eq!(
        candidate.text,
        format!(
            " \r\n[\n {} ,\t{}\r\n,  {} \n]\t",
            rows[1], rows[2], rows[0]
        )
    );
    assert_eq!(source, original);
    let reopened = DataTableSourceAdapter::read(
        &candidate,
        Some(&ReflectTypeInfo::struct_with_fields(fields)),
        "Name",
    )
    .unwrap();
    assert_eq!(
        reopened.rows[2].cells[0].value,
        ReflectedValue::Unsigned(u64::MAX)
    );
    let ReflectedValue::Scalar(value) = &reopened.rows[2].cells[1].value else {
        panic!("f32 expected")
    };
    assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(
        candidate.canonical_json,
        serde_json::from_str::<serde_json::Value>(&candidate.text).unwrap()
    );
}

#[test]
fn data_table_row_move_self_target_is_a_validated_noop_with_original_cache_untouched() {
    let fields = vec![field("v", "u64")];
    for text in [
        r#"[{"Name":"a","v":1}]"#,
        r#"[ {"Name":"a","v":1}, {"Name":"b","v":2} ]"#,
    ] {
        for cache_present in [false, true] {
            let mut source = asset(text);
            if cache_present {
                source.canonical_json = serde_json::from_str(text).unwrap();
            }
            let original = source.clone();
            assert_eq!(move_row(&source, fields.clone(), "a", "a"), Ok(None));
            assert_eq!(source, original);
        }
    }
}

#[test]
fn data_table_row_move_rejects_missing_source_or_target_even_in_empty_source() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let original = source.clone();
    for (key, target, missing) in [("missing", "a", "missing"), ("a", "missing", "missing")] {
        assert_eq!(
            move_row(&source, vec![field("v", "u64")], key, target),
            Err(DataTableCellError::MissingRow(missing.into()))
        );
    }
    assert_eq!(source, original);
    assert_eq!(
        move_row(&asset(" \n[]\t"), vec![field("v", "u64")], "a", "a"),
        Err(DataTableCellError::MissingRow("a".into()))
    );
}

#[test]
fn data_table_row_move_preflights_unselected_bad_cells_and_duplicate_properties_or_keys() {
    for text in [
        r#"[{"Name":"a","v":1},{"Name":"bad","v":"wrong"}]"#,
        r#"[{"Name":"a","v":1},{"Name":"bad","v":2,"v":3}]"#,
        r#"[{"Name":"a","v":1},{"Name":"a","v":2}]"#,
        r#"[{"Name":"a","v":1},{"Name":"bad","v":2,"unknown":0}]"#,
    ] {
        let source = asset(text);
        let original = source.clone();
        assert!(move_row(&source, vec![field("v", "u64")], "a", "a").is_err());
        assert_eq!(source, original);
    }
}

#[test]
fn data_table_row_move_rejects_missing_schema_wrong_format_and_stale_cache_before_noop() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"b","v":2}]"#);
    assert_eq!(
        DataTableCellCodec::prepare_row_move(&source, None, "Name", "a", "a"),
        Err(DataTableSourceError::MissingRowType.into())
    );
    let mut wrong_format = source.clone();
    wrong_format.format = DataAssetFormat::Toml;
    assert_eq!(
        move_row(&wrong_format, vec![field("v", "u64")], "a", "a"),
        Err(DataTableSourceError::UnsupportedFormat(DataAssetFormat::Toml).into())
    );
    let mut stale = source.clone();
    stale.canonical_json = json!([{"Name":"b","v":2},{"Name":"a","v":1}]);
    assert_eq!(
        move_row(&stale, vec![field("v", "u64")], "a", "a"),
        Err(DataTableSourceError::CanonicalMismatch.into())
    );
}

#[test]
fn data_table_row_move_exact_case_unicode_and_custom_escaped_key_never_alias() {
    let source = asset(r#"[{"I\u0064":"Row"},{"Id":"row"},{"Id":" 行 🧩 "}]"#);
    let row_type = ReflectTypeInfo::struct_with_fields(vec![]);
    let candidate =
        DataTableCellCodec::prepare_row_move(&source, Some(&row_type), "Id", " 行 🧩 ", "Row")
            .unwrap()
            .unwrap();
    assert_eq!(
        candidate.text,
        r#"[{"Id":" 行 🧩 "},{"I\u0064":"Row"},{"Id":"row"}]"#
    );
    for (key, target) in [("ROW", "Row"), ("Row", "行 🧩")] {
        assert!(matches!(
            DataTableCellCodec::prepare_row_move(&source, Some(&row_type), "Id", key, target),
            Err(DataTableCellError::MissingRow(_))
        ));
    }
}

#[test]
fn data_table_row_move_keeps_readonly_and_hidden_columns_without_value_edit_admission() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"b","v":2}]"#);
    let fields = vec![field("v", "u64")
        .with_editable(false)
        .with_editor_visible(false)];
    let candidate = move_row(&source, fields, "a", "b").unwrap().unwrap();
    assert_eq!(candidate.text, r#"[{"Name":"b","v":2},{"Name":"a","v":1}]"#);
}

#[test]
fn data_table_row_move_inverse_restores_source_text_but_host_must_keep_full_cache_inverse() {
    let source = asset(
        " \n[ {\"Name\":\"a\",\"v\":1},\t{\"Name\":\"b\",\"v\":2},\r\n{\"Name\":\"c\",\"v\":3} ]\t",
    );
    let original = source.clone();
    let fields = vec![field("v", "u64")];
    let moved = move_row(&source, fields.clone(), "a", "c")
        .unwrap()
        .unwrap();
    let restored = move_row(&moved, fields, "a", "b").unwrap().unwrap();
    assert_eq!(restored.text, original.text);
    assert_eq!(restored.uri, original.uri);
    assert_eq!(restored.format, original.format);
    assert!(original.canonical_json.is_null());
    assert!(!restored.canonical_json.is_null());
    assert_eq!(source, original);
}
