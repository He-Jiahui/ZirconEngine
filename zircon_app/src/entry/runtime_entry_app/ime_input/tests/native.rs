use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseKind, UiTextByteRange,
};

use super::*;

#[test]
fn imm_cursor_units_are_checked_and_converted_to_utf8_bytes() {
    assert_eq!(utf16_units_to_utf8_bytes("你🙂", 0), Some(0));
    assert_eq!(utf16_units_to_utf8_bytes("你🙂", 1), Some(3));
    assert_eq!(utf16_units_to_utf8_bytes("你🙂", 3), Some(7));
    assert_eq!(utf16_units_to_utf8_bytes("你🙂", 2), None);
    assert_eq!(utf16_units_to_utf8_bytes("你🙂", 4), None);
}

#[test]
fn imm_attributes_coalesce_utf8_safe_clause_ranges() {
    let text = "你a🙂";
    let clauses = clauses_from_attributes(text, &[2, 2, 3, 3]).unwrap();
    assert_eq!(
        clauses,
        vec![
            UiImePreeditClause::new(
                UiTextByteRange::new(0, 4),
                UiImePreeditClauseKind::Converted,
            ),
            UiImePreeditClause::new(
                UiTextByteRange::new(4, 8),
                UiImePreeditClauseKind::TargetNotConverted,
            ),
        ]
    );
}

#[test]
fn imm_attributes_reject_a_surrogate_pair_split_across_clauses() {
    assert_eq!(clauses_from_attributes("🙂", &[2, 3]), None);
}

#[test]
fn imm_attributes_reject_an_unbounded_clause_envelope() {
    let text = "a".repeat(MAX_NATIVE_IME_CLAUSES + 1);
    let attributes = (0..text.len())
        .map(|index| if index % 2 == 0 { 1 } else { 2 })
        .collect::<Vec<_>>();
    assert_eq!(clauses_from_attributes(&text, &attributes), None);
}
