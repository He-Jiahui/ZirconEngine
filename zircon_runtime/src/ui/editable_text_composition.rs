use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    component::UiValue,
    surface::{UiTextByteRange, UiTextPreeditClause, UiTextPreeditClauseKind},
    tree::UiTemplateNodeMetadata,
};

const COMPOSITION_CLAUSES_ATTRIBUTE: &str = "composition_clauses";

pub(in crate::ui) fn composition_clauses_from_metadata(
    metadata: &UiTemplateNodeMetadata,
    composition_text: &str,
) -> Vec<UiTextPreeditClause> {
    let Some(values) = metadata
        .attributes
        .get(COMPOSITION_CLAUSES_ATTRIBUTE)
        .and_then(toml::Value::as_array)
    else {
        return Vec::new();
    };
    validated_composition_clauses(
        values.iter().map(composition_clause_from_toml).collect(),
        composition_text,
    )
}

pub(in crate::ui) fn composition_clauses_from_value(
    value: Option<&UiValue>,
    composition_text: &str,
) -> Vec<UiTextPreeditClause> {
    let Some(UiValue::Array(values)) = value else {
        return Vec::new();
    };
    validated_composition_clauses(
        values.iter().map(composition_clause_from_value).collect(),
        composition_text,
    )
}

pub(in crate::ui) fn composition_clauses_value(clauses: &[UiTextPreeditClause]) -> UiValue {
    UiValue::Array(
        clauses
            .iter()
            .map(|clause| {
                UiValue::Map(BTreeMap::from([
                    (
                        "start_byte".to_string(),
                        UiValue::Int(clause.range.start_byte as i64),
                    ),
                    (
                        "end_byte".to_string(),
                        UiValue::Int(clause.range.end_byte as i64),
                    ),
                    (
                        "kind".to_string(),
                        UiValue::Enum(clause.kind.as_str().to_string()),
                    ),
                ]))
            })
            .collect(),
    )
}

fn validated_composition_clauses(
    clauses: Option<Vec<UiTextPreeditClause>>,
    composition_text: &str,
) -> Vec<UiTextPreeditClause> {
    clauses
        .filter(|clauses| {
            UiTextPreeditClause::validate_preedit_payload(composition_text, None, clauses).is_ok()
        })
        .unwrap_or_default()
}

fn composition_clause_from_toml(value: &toml::Value) -> Option<UiTextPreeditClause> {
    let table = value.as_table()?;
    composition_clause(
        table
            .get("start_byte")
            .and_then(toml::Value::as_integer)
            .and_then(|value| u32::try_from(value).ok())?,
        table
            .get("end_byte")
            .and_then(toml::Value::as_integer)
            .and_then(|value| u32::try_from(value).ok())?,
        table.get("kind").and_then(toml::Value::as_str)?,
    )
}

fn composition_clause_from_value(value: &UiValue) -> Option<UiTextPreeditClause> {
    let UiValue::Map(table) = value else {
        return None;
    };
    composition_clause(
        table.get("start_byte").and_then(ui_value_u32)?,
        table.get("end_byte").and_then(ui_value_u32)?,
        table.get("kind").and_then(ui_value_text)?,
    )
}

fn composition_clause(start_byte: u32, end_byte: u32, kind: &str) -> Option<UiTextPreeditClause> {
    Some(UiTextPreeditClause::new(
        UiTextByteRange::new(start_byte, end_byte),
        UiTextPreeditClauseKind::from_name(kind)?,
    ))
}

fn ui_value_u32(value: &UiValue) -> Option<u32> {
    match value {
        UiValue::Int(value) => u32::try_from(*value).ok(),
        _ => None,
    }
}

fn ui_value_text(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
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
}
