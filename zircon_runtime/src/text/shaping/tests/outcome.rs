use super::*;
use crate::text::shaping::{
    TextShapingFailureCode, TextShapingFailureDependency, TextShapingFailureDisposition,
    TextShapingFailurePhase, TextShapingFailureReceipt,
};
use crate::text::{FontFaceId, TextRange};

#[test]
fn request_diagnostics_are_fixed_and_stay_outside_the_shaped_cache_artifact() {
    assert!(
        std::mem::size_of::<TextShapingRequestDiagnostics>() <= 160,
        "request diagnostics must remain a small fixed-cardinality transient envelope"
    );
    for source in [
        include_str!("../../model/shaped_run.rs"),
        include_str!("../../cache/shaped_cache/memory.rs"),
    ] {
        assert!(!source.contains("TextShapingRequestDiagnostics"));
        assert!(!source.contains("TextFontResolutionReport"));
    }
}

#[test]
fn classifies_generation_changes_as_deferred_and_other_errors_as_failed() {
    assert!(matches!(
        TextShapingOutcome::<()>::from_result(Err(TextLayoutError::FontGenerationChanged)),
        TextShapingOutcome::Deferred(failure)
            if failure.error() == &TextLayoutError::FontGenerationChanged
                && failure.receipt().is_some_and(|receipt|
                    receipt.code == TextShapingFailureCode::FontGenerationChanged
                        && receipt.phase == TextShapingFailurePhase::FontResolution
                        && receipt.dependency == TextShapingFailureDependency::FontDatabase
                        && receipt.disposition == TextShapingFailureDisposition::Deferred)
    ));
    assert!(matches!(
        TextShapingOutcome::<()>::from_result(Err(TextLayoutError::BidiInvariant)),
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::BidiInvariant
    ));
}

#[test]
fn maps_only_ready_values_without_losing_error_disposition() {
    assert_eq!(
        TextShapingOutcome::from_result(Ok(2_u32)).map(|value| value * 2),
        TextShapingOutcome::Ready(4)
    );
    assert_eq!(
        TextShapingOutcome::<u32>::from_result(Err(TextLayoutError::FontGenerationChanged))
            .map(|value| value * 2),
        TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged)
    );
}

#[test]
fn request_failure_receipt_survives_outcome_transforms() {
    let receipt = TextShapingFailureReceipt {
        code: TextShapingFailureCode::BackendFaceParse,
        phase: TextShapingFailurePhase::FontLoad,
        source_range: Some(TextRange { start: 4, end: 9 }),
        face: Some(FontFaceId(7)),
        dependency: TextShapingFailureDependency::FontFace,
        disposition: TextShapingFailureDisposition::AlternateBackend,
        budget: None,
    };
    let outcome =
        TextShapingOutcome::<u32>::failed_with_receipt(TextLayoutError::ShapingFailed, receipt)
            .map(|value| value * 2)
            .and_then(|value| TextShapingOutcome::Ready(value + 1));

    assert_eq!(outcome.failure_receipt(), Some(receipt));
    assert_eq!(outcome.into_result(), Err(TextLayoutError::ShapingFailed));
}

#[test]
fn neutral_layout_failure_does_not_fabricate_a_shaping_receipt() {
    let outcome = TextShapingOutcome::<()>::failed(TextLayoutError::InvalidFontSize);

    assert_eq!(outcome.failure_receipt(), None);
    assert_eq!(outcome.into_result(), Err(TextLayoutError::InvalidFontSize));
}
