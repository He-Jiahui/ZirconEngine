//! In-memory candidate preparation; no project asset, schema, history or save producer.

use serde_json::json;
use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::{
    ReflectFieldInfo, ReflectNumericRange, ReflectTypeInfo, ReflectedValue,
};

use super::super::{
    DataTableCellCodec, DataTableCellEdit, DataTableCellError, DataTableSourceAdapter,
    DataTableSourceError,
};
use super::{asset, field};

fn edit(
    key: &str,
    field: &ReflectFieldInfo,
    expected: ReflectedValue,
    value: ReflectedValue,
) -> DataTableCellEdit {
    DataTableCellEdit {
        row_key: key.into(),
        field_id: field.id,
        expected,
        value,
    }
}

fn prepare(
    asset: &DataAsset,
    columns: Vec<ReflectFieldInfo>,
    edits: &[DataTableCellEdit],
) -> Result<Option<DataAsset>, DataTableCellError> {
    DataTableCellCodec::prepare(
        asset,
        Some(&ReflectTypeInfo::struct_with_fields(columns)),
        "Name",
        edits,
    )
}

#[test]
fn data_table_cells_preserve_every_unedited_utf8_escape_and_whitespace_byte() {
    let source = asset(
        "[\n  { \"Na\\u006de\":\"行二\",\"count\": 18446744073709551615, \"label\":\"\\u4e2d文\", \"ratio\":1.2500 }\n]\n",
    );
    let original = source.clone();
    let label = field("label", "alloc::string::String");
    let candidate = prepare(
        &source,
        vec![field("count", "u64"), label.clone(), field("ratio", "f32")],
        &[edit(
            "行二",
            &label,
            ReflectedValue::String("中文".into()),
            ReflectedValue::String("更长\n\"文字\"".into()),
        )],
    )
    .unwrap()
    .unwrap();
    assert_eq!(candidate.text, "[\n  { \"Na\\u006de\":\"行二\",\"count\": 18446744073709551615, \"label\":\"更长\\n\\\"文字\\\"\", \"ratio\":1.2500 }\n]\n");
    assert_eq!(candidate.uri, source.uri);
    assert_eq!(candidate.format, source.format);
    assert_eq!(source, original);
    assert_eq!(candidate.canonical_json[0]["label"], "更长\n\"文字\"");
}

#[test]
fn data_table_cells_batch_uses_original_keys_and_stable_fields_not_display_indices() {
    let source = asset(r#"[{"Name":"z","v":1,"text":"先"},{"Name":"a","v":2,"text":"后"}]"#);
    let v = field("v", "u64").with_display_name("排序后的显示列");
    let text = field("text", "alloc::string::String");
    let candidate = prepare(
        &source,
        vec![text.clone(), v.clone()],
        &[
            edit(
                "a",
                &v,
                ReflectedValue::Unsigned(2),
                ReflectedValue::Unsigned(u64::MAX),
            ),
            edit(
                "z",
                &text,
                ReflectedValue::String("先".into()),
                ReflectedValue::String("变长的第一行".into()),
            ),
            edit(
                "z",
                &v,
                ReflectedValue::Unsigned(1),
                ReflectedValue::Unsigned(3),
            ),
        ],
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        candidate.text,
        r#"[{"Name":"z","v":3,"text":"变长的第一行"},{"Name":"a","v":18446744073709551615,"text":"后"}]"#
    );
}

#[test]
fn data_table_cells_noop_retains_noncanonical_number_spelling_and_unset_cache() {
    let source = asset(" \n[{\"Name\":\"a\", \"v\":1.2500}]\t");
    let v = field("v", "f32");
    assert_eq!(
        prepare(
            &source,
            vec![v.clone()],
            &[edit(
                "a",
                &v,
                ReflectedValue::Scalar(1.25),
                ReflectedValue::Scalar(1.25)
            )]
        )
        .unwrap(),
        None
    );
    assert_eq!(prepare(&source, vec![v], &[]).unwrap(), None);
    assert!(source.text.ends_with("1.2500}]\t"));
    assert!(source.canonical_json.is_null());
}

#[test]
fn data_table_cells_reject_final_stale_value_without_returning_partial_candidate() {
    let source = asset(r#"[{"Name":"a","v":1},{"Name":"b","v":2}]"#);
    let original = source.clone();
    let v = field("v", "u64");
    let result = prepare(
        &source,
        vec![v.clone()],
        &[
            edit(
                "a",
                &v,
                ReflectedValue::Unsigned(1),
                ReflectedValue::Unsigned(3),
            ),
            edit(
                "b",
                &v,
                ReflectedValue::Unsigned(99),
                ReflectedValue::Unsigned(4),
            ),
        ],
    );
    assert!(
        matches!(result, Err(DataTableCellError::ExpectedValueMismatch { row, .. }) if row == "b")
    );
    assert_eq!(source, original);
}

#[test]
fn data_table_cells_reject_duplicate_edits_including_chained_expected_values() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let v = field("v", "u64");
    assert!(matches!(
        prepare(
            &source,
            vec![v.clone()],
            &[
                edit(
                    "a",
                    &v,
                    ReflectedValue::Unsigned(1),
                    ReflectedValue::Unsigned(2)
                ),
                edit(
                    "a",
                    &v,
                    ReflectedValue::Unsigned(2),
                    ReflectedValue::Unsigned(3)
                ),
            ]
        ),
        Err(DataTableCellError::DuplicateEdit { .. })
    ));
}

#[test]
fn data_table_cells_reject_missing_rows_unknown_fields_and_noneditable_schema() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let v = field("v", "u64");
    let base = edit(
        "a",
        &v,
        ReflectedValue::Unsigned(1),
        ReflectedValue::Unsigned(2),
    );
    let mut missing = base.clone();
    missing.row_key = "missing".into();
    assert_eq!(
        prepare(&source, vec![v.clone()], &[missing]),
        Err(DataTableCellError::MissingRow("missing".into()))
    );
    let mut unknown = base.clone();
    unknown.field_id = field("other", "u64").id;
    assert!(matches!(
        prepare(&source, vec![v.clone()], &[unknown]),
        Err(DataTableCellError::UnknownField(_))
    ));
    for column in [v.clone().with_editable(false), v.with_editor_visible(false)] {
        assert!(matches!(
            prepare(&source, vec![column], &[base.clone()]),
            Err(DataTableCellError::FieldNotEditable(_))
        ));
    }
}

#[test]
fn data_table_cells_preserve_integer_width_and_variant_without_float_coercion() {
    for (path, old, new, expected) in [
        (
            "u64",
            ReflectedValue::Unsigned(0),
            ReflectedValue::Unsigned(u64::MAX),
            "18446744073709551615",
        ),
        (
            "i64",
            ReflectedValue::Integer(0),
            ReflectedValue::Integer(i64::MIN),
            "-9223372036854775808",
        ),
    ] {
        let v = field("v", path);
        let source = asset(r#"[{"Name":"a","v":0}]"#);
        let candidate = prepare(&source, vec![v.clone()], &[edit("a", &v, old, new)])
            .unwrap()
            .unwrap();
        assert_eq!(
            candidate.text,
            format!(r#"[{{"Name":"a","v":{expected}}}]"#)
        );
    }
    let v = field("v", "u8");
    for value in [
        ReflectedValue::Unsigned(256),
        ReflectedValue::Integer(1),
        ReflectedValue::Scalar(1.0),
        ReflectedValue::String("1".into()),
    ] {
        assert!(matches!(
            prepare(
                &asset(r#"[{"Name":"a","v":0}]"#),
                vec![v.clone()],
                &[edit("a", &v, ReflectedValue::Unsigned(0), value)]
            ),
            Err(DataTableCellError::InvalidValue { .. })
        ));
    }
}

#[test]
fn data_table_cells_check_integer_range_at_u64_and_i64_float_boundaries_exactly() {
    for (path, limit, value, should_pass) in [
        (
            "u64",
            ReflectNumericRange::new(None, Some(2.0_f32.powi(64)), None, None).unwrap(),
            ReflectedValue::Unsigned(u64::MAX),
            true,
        ),
        (
            "u64",
            ReflectNumericRange::new(Some(2.0_f32.powi(64)), None, None, None).unwrap(),
            ReflectedValue::Unsigned(u64::MAX),
            false,
        ),
        (
            "i64",
            ReflectNumericRange::new(Some(2.0_f32.powi(63)), None, None, None).unwrap(),
            ReflectedValue::Integer(i64::MAX),
            false,
        ),
        (
            "i64",
            ReflectNumericRange::new(None, Some(-2.0_f32.powi(63)), None, None).unwrap(),
            ReflectedValue::Integer(i64::MIN),
            true,
        ),
    ] {
        let v = field("v", path).with_numeric_range(limit);
        let old = if path == "u64" {
            ReflectedValue::Unsigned(0)
        } else {
            ReflectedValue::Integer(0)
        };
        let result = prepare(
            &asset(r#"[{"Name":"a","v":0}]"#),
            vec![v.clone()],
            &[edit("a", &v, old, value)],
        );
        if should_pass {
            assert!(result.unwrap().is_some());
        } else {
            assert!(matches!(
                result,
                Err(DataTableCellError::InvalidValue { .. })
            ));
        }
    }
}

#[test]
fn data_table_cells_integer_ranges_apply_ceil_floor_and_handle_extreme_float_limits() {
    let range = ReflectNumericRange::new(Some(-3.5), Some(12.25), Some(2.0), Some(0)).unwrap();
    for (value, valid) in [(-4, false), (-3, true), (12, true), (13, false)] {
        let v = field("v", "i64").with_numeric_range(range.clone());
        let result = prepare(
            &asset(r#"[{"Name":"a","v":0}]"#),
            vec![v.clone()],
            &[edit(
                "a",
                &v,
                ReflectedValue::Integer(0),
                ReflectedValue::Integer(value),
            )],
        );
        assert_eq!(result.is_ok(), valid);
    }
    let v = field("v", "u64").with_numeric_range(
        ReflectNumericRange::new(Some(-f32::MAX), Some(f32::MAX), None, None).unwrap(),
    );
    assert!(prepare(
        &asset(r#"[{"Name":"a","v":0}]"#),
        vec![v.clone()],
        &[edit(
            "a",
            &v,
            ReflectedValue::Unsigned(0),
            ReflectedValue::Unsigned(u64::MAX)
        )]
    )
    .unwrap()
    .is_some());
}

#[test]
fn data_table_cells_reject_nonfinite_scalar_and_respect_actual_range_without_rounding() {
    let v = field("v", "f32").with_numeric_range(
        ReflectNumericRange::new(Some(0.0), Some(1.0), Some(0.5), Some(0)).unwrap(),
    );
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.1] {
        assert!(matches!(
            prepare(
                &asset(r#"[{"Name":"a","v":0}]"#),
                vec![v.clone()],
                &[edit(
                    "a",
                    &v,
                    ReflectedValue::Scalar(0.0),
                    ReflectedValue::Scalar(value)
                )]
            ),
            Err(DataTableCellError::InvalidValue { .. })
        ));
    }
    let candidate = prepare(
        &asset(r#"[{"Name":"a","v":0}]"#),
        vec![v.clone()],
        &[edit(
            "a",
            &v,
            ReflectedValue::Scalar(0.0),
            ReflectedValue::Scalar(0.125),
        )],
    )
    .unwrap()
    .unwrap();
    assert_eq!(candidate.text, r#"[{"Name":"a","v":0.125}]"#);
}

#[test]
fn data_table_cells_distinguish_signed_zero_expectations_and_deliberate_changes() {
    let source = asset(r#"[{"Name":"a","v":-0.0}]"#);
    let v = field("v", "f32");
    assert!(matches!(
        prepare(
            &source,
            vec![v.clone()],
            &[edit(
                "a",
                &v,
                ReflectedValue::Scalar(0.0),
                ReflectedValue::Scalar(1.0)
            )]
        ),
        Err(DataTableCellError::ExpectedValueMismatch { .. })
    ));
    let candidate = prepare(
        &source,
        vec![v.clone()],
        &[edit(
            "a",
            &v,
            ReflectedValue::Scalar(-0.0),
            ReflectedValue::Scalar(0.0),
        )],
    )
    .unwrap()
    .unwrap();
    let reopened = DataTableSourceAdapter::read(
        &candidate,
        Some(&ReflectTypeInfo::struct_with_fields(vec![v])),
        "Name",
    )
    .unwrap();
    let ReflectedValue::Scalar(value) = &reopened.rows[0].cells[0].value else {
        panic!("expected f32 cell");
    };
    assert_eq!(value.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn data_table_cells_require_actual_type_and_consistent_original_cache() {
    let source = asset(r#"[{"Name":"a","v":1}]"#);
    let v = field("v", "u64");
    let change = edit(
        "a",
        &v,
        ReflectedValue::Unsigned(1),
        ReflectedValue::Unsigned(2),
    );
    assert_eq!(
        DataTableCellCodec::prepare(&source, None, "Name", &[change.clone()]),
        Err(DataTableCellError::Source(
            DataTableSourceError::MissingRowType
        ))
    );
    let mut stale = source.clone();
    stale.canonical_json = json!([{"Name":"a","v":3}]);
    assert_eq!(
        prepare(&stale, vec![v], &[change]),
        Err(DataTableCellError::Source(
            DataTableSourceError::CanonicalMismatch
        ))
    );
}

#[test]
fn data_table_cells_reuse_duplicate_unknown_and_unsupported_source_rejection() {
    let v = field("v", "u64");
    let change = edit(
        "a",
        &v,
        ReflectedValue::Unsigned(1),
        ReflectedValue::Unsigned(2),
    );
    for text in [
        r#"[{"Name":"a","v":1,"v":2}]"#,
        r#"[{"Name":"a","v":1,"extra":42}]"#,
        r#"[{"Name":"a"}]"#,
    ] {
        assert!(matches!(
            prepare(&asset(text), vec![v.clone()], &[change.clone()]),
            Err(DataTableCellError::Source(_))
        ));
    }
    assert!(matches!(
        prepare(&source_for_unsupported(), vec![field("v", "f64")], &[]),
        Err(DataTableCellError::Source(
            DataTableSourceError::UnsupportedColumn { .. }
        ))
    ));
}

fn source_for_unsupported() -> DataAsset {
    asset(r#"[{"Name":"a","v":1}]"#)
}

#[test]
fn data_table_cells_use_explicit_import_key_and_reject_row_key_as_editable_column() {
    let v = field("v", "u64");
    let row_type = ReflectTypeInfo::struct_with_fields(vec![v.clone()]);
    let source = asset(r#"[{"行键":"原行","v":1}]"#);
    let candidate = DataTableCellCodec::prepare(
        &source,
        Some(&row_type),
        "行键",
        &[edit(
            "原行",
            &v,
            ReflectedValue::Unsigned(1),
            ReflectedValue::Unsigned(2),
        )],
    )
    .unwrap()
    .unwrap();
    assert_eq!(candidate.text, r#"[{"行键":"原行","v":2}]"#);
    let key = field("行键", "alloc::string::String");
    let invalid = ReflectTypeInfo::struct_with_fields(vec![v, key]);
    assert!(matches!(
        DataTableCellCodec::prepare(&source, Some(&invalid), "行键", &[]),
        Err(DataTableCellError::Source(
            DataTableSourceError::InvalidRowType(_)
        ))
    ));
}
