use super::*;

fn context(
    window_generation: u64,
    focus_generation: u64,
    composition_generation: u64,
) -> ZrRuntimeImeCompositionContextV2 {
    ZrRuntimeImeCompositionContextV2 {
        window_generation,
        focus_generation,
        composition_generation,
    }
}

fn preedit() -> ZrRuntimeImeCompositionV2 {
    ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Preedit,
        context: context(8, 4, 12),
        text: "n你🙂".to_string(),
        cursor_range: Some(UiTextByteRange::new(4, 8)),
        clauses: vec![
            UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
            UiImePreeditClause::new(
                UiTextByteRange::new(1, 4),
                UiImePreeditClauseKind::Converted,
            ),
            UiImePreeditClause::new(
                UiTextByteRange::new(4, 8),
                UiImePreeditClauseKind::TargetConverted,
            ),
        ],
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Available,
    }
}

#[test]
fn v2_round_trip_preserves_utf8_clause_boundaries_and_context_generations() {
    let original = preedit();
    let payload = original.encode().expect("encode bounded preedit");
    assert_eq!(ZrRuntimeImeCompositionV2::decode(&payload), Ok(original));
}

#[test]
fn v2_rejects_clause_boundaries_inside_a_multibyte_scalar() {
    let mut malformed = preedit();
    malformed.clauses[1].range.start_byte = 2;
    assert!(matches!(
        malformed.encode(),
        Err(ZrRuntimeImeCompositionV2Error::InvalidPreeditRange(
            UiImePreeditClauseError::RangeNotUtf8Boundary
        ))
    ));
}

#[test]
fn v2_rejects_truncated_payloads_unknown_kinds_and_nonzero_reserved_fields() {
    let payload = preedit().encode().expect("valid payload");
    assert_eq!(
        ZrRuntimeImeCompositionV2::decode(&payload[..payload.len() - 1]),
        Err(ZrRuntimeImeCompositionV2Error::LengthMismatch)
    );

    let mut unknown_kind = payload.clone();
    unknown_kind[ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES + 8] = 99;
    assert_eq!(
        ZrRuntimeImeCompositionV2::decode(&unknown_kind),
        Err(ZrRuntimeImeCompositionV2Error::UnknownClauseKind)
    );

    let mut unknown_availability = preedit().encode().expect("valid payload");
    unknown_availability[48..52].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        ZrRuntimeImeCompositionV2::decode(&unknown_availability),
        Err(ZrRuntimeImeCompositionV2Error::UnknownClauseAvailability)
    );

    let mut reserved = payload;
    reserved[ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES + 10] = 1;
    assert_eq!(
        ZrRuntimeImeCompositionV2::decode(&reserved),
        Err(ZrRuntimeImeCompositionV2Error::NonZeroReservedClauseField)
    );
}

#[test]
fn v2_cancel_payload_cannot_carry_text_cursor_or_clause_data() {
    let malformed = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Cancel,
        context: context(8, 5, 12),
        text: "stale".to_string(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    };
    assert_eq!(
        malformed.encode(),
        Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload)
    );
}
