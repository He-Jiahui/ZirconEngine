//! In-memory exchange cases only; these never register an asset, row schema or provider.

use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{
    ReflectFieldInfo, ReflectNumericRange, ReflectTypeInfo, ReflectedValue,
};

use super::super::{
    DataTableCellError, DataTableCsvCodec, DataTableCsvError, DataTableSourceAdapter,
    DataTableSourceError,
};
use super::{asset, field};

fn import(
    source: &DataAsset,
    fields: Vec<ReflectFieldInfo>,
    csv: &str,
) -> Result<Option<DataAsset>, DataTableCsvError> {
    DataTableCsvCodec::prepare_reimport(
        source,
        Some(&ReflectTypeInfo::struct_with_fields(fields)),
        "Name",
        csv,
    )
}

#[test]
fn data_table_csv_round_trip_keeps_source_bytes_integer_limits_unicode_and_quoted_crlf() {
    let source = asset(
        " [ {\"Name\":\"行,\\\"二\\\"\",\"count\":18446744073709551615,\"delta\":-9223372036854775808,\"ratio\":-0.0,\"text\":\"中文,\\\"引号\\\"\\r\\n下一行\",\"on\":false} ]\n",
    );
    let original = source.clone();
    let row_type = ReflectTypeInfo::struct_with_fields(vec![
        field("count", "u64"),
        field("delta", "i64"),
        field("ratio", "f32"),
        field("text", "alloc::string::String"),
        field("on", "bool"),
    ]);
    let csv = DataTableCsvCodec::encode(&source, Some(&row_type), "Name").unwrap();
    assert!(csv.starts_with("Name,count,delta,ratio,text,on\r\n"));
    assert!(csv.contains("18446744073709551615,-9223372036854775808,-0.0"));
    assert!(csv.contains("\"中文,\"\"引号\"\"\r\n下一行\""));
    assert_eq!(
        DataTableCsvCodec::prepare_reimport(&source, Some(&row_type), "Name", &csv).unwrap(),
        None
    );
    assert_eq!(source, original);
    assert!(source.canonical_json.is_null());
}

#[test]
fn data_table_csv_full_replacement_uses_csv_row_order_and_named_columns() {
    let source = asset(r#"[{"Name":"old","v":1,"text":"旧"}]"#);
    let original = source.clone();
    let row_type = ReflectTypeInfo::struct_with_fields(vec![
        field("v", "u64").with_display_name("显示列名"),
        field("text", "alloc::string::String"),
    ]);
    let candidate = DataTableCsvCodec::prepare_reimport(
        &source,
        Some(&row_type),
        "Name",
        "text,Name,v\n末行,z,18446744073709551615\n首行,a,9007199254740993\n",
    )
    .unwrap()
    .unwrap();
    let reopened = DataTableSourceAdapter::read(&candidate, Some(&row_type), "Name").unwrap();
    assert_eq!(reopened.rows[0].source_key, "z");
    assert_eq!(reopened.rows[1].source_key, "a");
    assert_eq!(reopened.rows[0].cells[0].field_id, row_type.fields[0].id);
    assert_eq!(
        reopened.rows[0].cells[0].value,
        ReflectedValue::Unsigned(u64::MAX)
    );
    assert_eq!(
        reopened.rows[1].cells[0].value,
        ReflectedValue::Unsigned(9007199254740993)
    );
    assert_eq!(candidate.uri, source.uri);
    assert_eq!(candidate.format, DataAssetFormat::Json);
    assert_eq!(source, original);
}

#[test]
fn data_table_csv_header_only_clears_rows_and_no_defaults_are_synthesized() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let candidate = import(&source, vec![field("v", "u64")], "Name,v\r\n")
        .unwrap()
        .unwrap();
    assert_eq!(candidate.text, "[]");
    assert_eq!(
        import(&candidate, vec![field("v", "u64")], "Name,v").unwrap(),
        None
    );
    assert!(import(&source, vec![field("v", "u64")], "Name,v\nnew,\n").is_err());
    assert!(import(&source, vec![field("v", "u64")], "Name\nnew\n").is_err());
}

#[test]
fn data_table_csv_rejects_duplicate_missing_unknown_and_display_name_headers() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    for csv in [
        "Name,v,v\na,1,1",
        "Name\na",
        "v\n1",
        "Name,v,unknown\na,1,2",
        "Name,显示名\na,1",
        "Name,\na,1",
        "Name,V\na,1",
    ] {
        assert!(matches!(
            import(
                &source,
                vec![field("v", "u64").with_display_name("显示名")],
                csv
            ),
            Err(DataTableCsvError::Header(_))
        ));
    }
    assert_eq!(
        import(&source, vec![field("v", "u64")], ""),
        Err(DataTableCsvError::MissingHeader)
    );
}

#[test]
fn data_table_csv_all_records_preflight_before_candidate_and_original_stays_immutable() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let original = source.clone();
    for csv in [
        "Name,v\nfirst,2\nlate,3,extra",
        "Name,v\nfirst,2\nlate",
        "Name,v\nfirst,2\nfirst,3",
        "Name,v\nfirst,2\n,3",
        "Name,v\nfirst,2\nlate,18446744073709551616",
        "Name,v\nfirst,2\nlate,\"unterminated",
        "Name,v\nfirst,2\n\n",
    ] {
        assert!(
            import(&source, vec![field("v", "u64")], csv).is_err(),
            "{csv}"
        );
        assert_eq!(source, original);
    }
}

#[test]
fn data_table_csv_quote_and_record_syntax_are_strict_without_whitespace_recovery() {
    let source = asset("[]");
    for csv in [
        "Name,text\rkey,text",
        "Name,text\nkey,ba\"d",
        "Name,text\nkey,\"closed\"trailing",
        "Name,text\nkey,\"closed\" ,",
        "Name,text\nkey,\"unterminated",
    ] {
        assert!(
            matches!(
                import(&source, vec![field("text", "alloc::string::String")], csv),
                Err(DataTableCsvError::Syntax { .. })
            ),
            "{csv}"
        );
    }
    let candidate = import(
        &source,
        vec![field("text", "alloc::string::String")],
        "Name,text\r\n\"行,\\路径\",\"\"\"中文\"\"\r\n保留\r和LF\n\"\r\n",
    )
    .unwrap()
    .unwrap();
    assert_eq!(candidate.canonical_json[0]["Name"], "行,\\路径");
    assert_eq!(
        candidate.canonical_json[0]["text"],
        "\"中文\"\r\n保留\r和LF\n"
    );
}

#[test]
fn data_table_csv_readonly_and_hidden_cells_can_roundtrip_but_not_change_or_initialize() {
    let source = asset(r#"[{"Name":"a","v":1,"text":"locked"}]"#);
    for protected in [
        field("v", "u64").with_editable(false),
        field("v", "u64").with_editor_visible(false),
    ] {
        let fields = vec![protected.clone(), field("text", "alloc::string::String")];
        assert_eq!(
            import(&source, fields.clone(), "Name,v,text\na,1,locked").unwrap(),
            None
        );
        assert!(import(&source, fields.clone(), "Name,v,text\na,1,changed")
            .unwrap()
            .is_some());
        for csv in ["Name,v,text\na,2,locked", "Name,v,text\nnew,1,locked"] {
            assert!(matches!(import(&source, fields.clone(), csv),
                Err(DataTableCsvError::Cell(DataTableCellError::FieldNotEditable(id))) if id == protected.id
            ));
        }
    }
}

#[test]
fn data_table_csv_rejects_coercion_nonfinite_and_lossy_numeric_cells() {
    for (path, invalid) in [
        ("u64", "1.0"),
        ("u64", "1e0"),
        ("u64", "-1"),
        ("i8", "128"),
        ("u8", "256"),
        ("bool", "1"),
        ("f32", "NaN"),
        ("f32", "Infinity"),
        ("f32", "1e100"),
        ("f32", "1.000000000000000000000001"),
        ("f32", "16777217"),
    ] {
        let source = asset(if path == "bool" {
            r#"[{"Name":"a","v":false}]"#
        } else {
            r#"[{"Name":"a","v":0}]"#
        });
        assert!(
            matches!(
                import(
                    &source,
                    vec![field("v", path)],
                    &format!("Name,v\na,{invalid}")
                ),
                Err(DataTableCsvError::Cell(
                    DataTableCellError::InvalidValue { .. }
                ))
            ),
            "{path}/{invalid}"
        );
    }
}

#[test]
fn data_table_csv_preserves_signed_zero_and_never_rounds_to_precision_or_step() {
    let v = field("v", "f32").with_numeric_range(
        ReflectNumericRange::new(Some(-1.0), Some(1.0), Some(0.5), Some(0)).unwrap(),
    );
    let source = asset(r#"[{"Name":"a","v":0.0}]"#);
    let zero = import(&source, vec![v.clone()], "Name,v\na,-0.0")
        .unwrap()
        .unwrap();
    assert!(zero.text.contains("-0.0"));
    assert_eq!(
        import(&zero, vec![v.clone()], "Name,v\na,-0.000").unwrap(),
        None
    );
    let quarter = import(&source, vec![v.clone()], "Name,v\na,0.25")
        .unwrap()
        .unwrap();
    assert_eq!(quarter.canonical_json[0]["v"], 0.25);
    assert!(import(&source, vec![v], "Name,v\na,2").is_err());
}

#[test]
fn data_table_csv_requires_current_valid_source_schema_and_canonical_cache() {
    let source = asset("[]");
    assert_eq!(
        DataTableCsvCodec::prepare_reimport(&source, None, "Name", ""),
        Err(DataTableCsvError::Source(
            DataTableSourceError::MissingRowType
        ))
    );
    let mut wrong = asset(r#"[{"Name":"a","v":1}]"#);
    wrong.canonical_json = serde_json::json!([]);
    assert!(matches!(
        import(&wrong, vec![field("v", "u64")], "Name,v\na,2"),
        Err(DataTableCsvError::Source(
            DataTableSourceError::CanonicalMismatch
        ))
    ));
    wrong.format = DataAssetFormat::Toml;
    assert!(import(&wrong, vec![field("v", "u64")], "Name,v\na,2").is_err());
    assert!(import(&asset("[]"), vec![field("v", "f64")], "Name,v\na,1").is_err());
}

#[test]
fn data_table_csv_bom_unicode_case_whitespace_and_formula_text_keep_exact_keys() {
    let source = asset("[]");
    let candidate = import(
        &source,
        vec![field("text", "alloc::string::String")],
        "\u{feff}Name,text\nName,=1+1\nname,\" 空格 \"\n 无修剪 ,\"\"\"尾\"\"\"\n",
    )
    .unwrap()
    .unwrap();
    assert_eq!(candidate.canonical_json[0]["Name"], "Name");
    assert_eq!(candidate.canonical_json[1]["Name"], "name");
    assert_eq!(candidate.canonical_json[2]["Name"], " 无修剪 ");
    assert_eq!(candidate.canonical_json[0]["text"], "=1+1");
    assert_eq!(candidate.canonical_json[1]["text"], " 空格 ");
    assert_eq!(candidate.canonical_json[2]["text"], "\"尾\"");
}

#[test]
fn data_table_csv_export_retains_source_order_raw_number_tokens_and_serializable_fields() {
    let source =
        asset(r#"[{"Name":"z","ratio":1.2500,"text":""},{"Name":"a","ratio":-0.0,"text":"后"}]"#);
    let row_type = ReflectTypeInfo::struct_with_fields(vec![
        field("text", "alloc::string::String").with_editable(false),
        field("ratio", "f32").with_editor_visible(false),
    ]);
    let csv = DataTableCsvCodec::encode(&source, Some(&row_type), "Name").unwrap();
    assert_eq!(csv, "Name,text,ratio\r\nz,,1.2500\r\na,后,-0.0\r\n");
    assert_eq!(
        DataTableCsvCodec::prepare_reimport(&source, Some(&row_type), "Name", &csv).unwrap(),
        None
    );
}

#[test]
fn data_table_csv_unchanged_legacy_values_keep_tokens_without_reapplying_new_editor_bounds() {
    let source = asset(r#"[{"Name":"a","ratio":2.5000,"text":"old"}]"#);
    for editable in [false, true] {
        let row_type = ReflectTypeInfo::struct_with_fields(vec![
            field("ratio", "f32")
                .with_editable(editable)
                .with_numeric_range(
                    ReflectNumericRange::new(Some(-1.0), Some(1.0), None, None).unwrap(),
                ),
            field("text", "alloc::string::String"),
        ]);
        let exported = DataTableCsvCodec::encode(&source, Some(&row_type), "Name").unwrap();
        assert_eq!(
            DataTableCsvCodec::prepare_reimport(&source, Some(&row_type), "Name", &exported)
                .unwrap(),
            None
        );
        let candidate = DataTableCsvCodec::prepare_reimport(
            &source,
            Some(&row_type),
            "Name",
            "Name,ratio,text\na,2.5,changed",
        )
        .unwrap()
        .unwrap();
        assert!(candidate.text.contains("2.5000"));
        assert_eq!(candidate.canonical_json[0]["text"], "changed");
        assert!(DataTableCsvCodec::prepare_reimport(
            &source,
            Some(&row_type),
            "Name",
            "Name,ratio,text\na,3,old",
        )
        .is_err());
    }
}
