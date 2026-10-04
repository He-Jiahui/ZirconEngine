use super::*;

#[test]
fn component_value_round_trip_preserves_multibyte_preedit_clauses() {
    let clauses = vec![
        UiTextPreeditClause::new(UiTextByteRange::new(0, 1), UiTextPreeditClauseKind::Input),
        UiTextPreeditClause::new(
            UiTextByteRange::new(1, 4),
            UiTextPreeditClauseKind::Converted,
        ),
    ];
    let value = composition_clauses_value(&clauses);

    assert_eq!(
        composition_clauses_from_value(Some(&value), "a\u{754c}"),
        clauses
    );
}

#[test]
fn component_value_rejects_invalid_preedit_clause_payload() {
    let value = UiValue::Array(vec![UiValue::Map(BTreeMap::from([
        ("start_byte".to_string(), UiValue::Int(1)),
        ("end_byte".to_string(), UiValue::Int(2)),
        ("kind".to_string(), UiValue::Enum("input".to_string())),
    ]))]);

    assert!(composition_clauses_from_value(Some(&value), "\u{754c}").is_empty());
}
