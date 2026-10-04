use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseKind, UiInputSequence, UiInputTimestamp, UiTextByteRange,
};

use super::*;

#[test]
fn decoded_preedit_adapter_preserves_clause_ranges_kinds_and_utf8_text() {
    let clauses = vec![
        UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
        UiImePreeditClause::new(
            UiTextByteRange::new(1, 4),
            UiImePreeditClauseKind::Converted,
        ),
        UiImePreeditClause::new(
            UiTextByteRange::new(4, 8),
            UiImePreeditClauseKind::TargetConverted,
        ),
    ];
    let decoded = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Preedit,
        context: zircon_runtime_interface::ZrRuntimeImeCompositionContextV2 {
            window_generation: 3,
            focus_generation: 5,
            composition_generation: 9,
        },
        text: "n你🙂".to_string(),
        cursor_range: Some(UiTextByteRange::new(4, 8)),
        clauses: clauses.clone(),
        clause_availability:
            zircon_runtime_interface::ZrRuntimeImePreeditClauseAvailabilityV2::Available,
    };
    let metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(17), UiInputSequence::new(23));

    let UiInputEvent::Ime(actual) =
        decoded_preedit_to_ui_event(decoded, metadata).expect("preedit produces one UI IME event")
    else {
        panic!("adapter must return an IME event");
    };
    assert_eq!(actual.kind, UiImeInputEventKind::Preedit);
    assert_eq!(actual.text, "n你🙂");
    assert_eq!(actual.cursor_range, Some(UiTextByteRange::new(4, 8)));
    assert_eq!(actual.preedit_clauses, clauses);
    actual.validate().expect("adapted UI event is valid");
}

#[test]
fn commit_and_cancel_operations_do_not_masquerade_as_preedit() {
    for operation in [
        ZrRuntimeImeCompositionOperationV2::Commit,
        ZrRuntimeImeCompositionOperationV2::Cancel,
    ] {
        let decoded = ZrRuntimeImeCompositionV2 {
            operation,
            context: zircon_runtime_interface::ZrRuntimeImeCompositionContextV2 {
                window_generation: 3,
                focus_generation: 5,
                composition_generation: 9,
            },
            text: String::new(),
            cursor_range: None,
            clauses: Vec::new(),
            clause_availability:
                zircon_runtime_interface::ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
        };
        assert!(decoded_preedit_to_ui_event(decoded, UiInputEventMetadata::default()).is_none());
    }
}
