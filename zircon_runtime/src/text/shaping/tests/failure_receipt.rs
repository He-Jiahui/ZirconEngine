use std::path::PathBuf;

use crate::asset::assets::FontSourceBudgetError;
use crate::core::framework::text::TextLayoutError;
use crate::text::shaping::fallback_spans::FallbackItemizationError;
use crate::text::FontFaceId;

use super::*;

#[test]
fn backend_failure_receipt_retains_stable_face_range_and_policy() {
    let face = FontFaceId(7);
    let range = TextRange { start: 4, end: 9 };
    let error = DirectShapeError::backend(
        range,
        BackendShapeError::FaceParseFailed {
            face,
            face_index: 2,
        },
    );

    let horizontal = TextShapingFailureReceipt::from_direct(&error, TextOrientation::Horizontal);
    let vertical = TextShapingFailureReceipt::from_direct(&error, TextOrientation::Vertical);

    assert_eq!(horizontal.code, TextShapingFailureCode::BackendFaceParse);
    assert_eq!(horizontal.phase, TextShapingFailurePhase::FontLoad);
    assert_eq!(horizontal.source_range, Some(range));
    assert_eq!(horizontal.face, Some(face));
    assert!(horizontal.allows_alternate_backend());
    assert!(!vertical.allows_alternate_backend());
}

#[test]
fn invariant_and_budget_failures_are_terminal() {
    let bidi = DirectShapeError::from(BidiInvariantError::InvalidLineRange { start: 9, end: 4 });
    let bidi_receipt = TextShapingFailureReceipt::from_direct(&bidi, TextOrientation::Horizontal);
    assert_eq!(bidi_receipt.code, TextShapingFailureCode::BidiInvariant);
    assert_eq!(
        bidi_receipt.source_range,
        Some(TextRange { start: 9, end: 4 })
    );
    assert!(!bidi_receipt.allows_alternate_backend());

    let budget = DirectShapeError::backend(
        TextRange { start: 0, end: 4 },
        BackendShapeError::font_database(
            BackendFontOperation::LoadFaceBytes,
            FontFaceId(3),
            FontDatabaseError::SourceBudget {
                path: PathBuf::from("font.ttf"),
                source: FontSourceBudgetError::SourceBytes {
                    limit_bytes: 4,
                    actual_bytes: 5,
                },
            },
        ),
    );
    let budget_receipt =
        TextShapingFailureReceipt::from_direct(&budget, TextOrientation::Horizontal);
    assert_eq!(
        budget_receipt.code,
        TextShapingFailureCode::FontSourceBudgetExceeded
    );
    assert_eq!(
        budget_receipt.dependency,
        TextShapingFailureDependency::WorkBudget
    );
    assert_eq!(
        budget_receipt.budget,
        Some(TextShapingBudgetKind::FontSourceAdmission)
    );
    assert!(!budget_receipt.allows_alternate_backend());
}

#[test]
fn missing_primary_face_keeps_a_typed_font_resolution_receipt() {
    let failure = TextShapingFailure::from(FallbackItemizationError::PrimaryFaceUnavailable);
    let receipt = failure
        .receipt()
        .expect("missing primary face must retain its capability cause");

    assert_eq!(failure.error(), &TextLayoutError::FontUnavailable);
    assert_eq!(receipt.code, TextShapingFailureCode::FontPrimaryUnavailable);
    assert_eq!(receipt.phase, TextShapingFailurePhase::FontResolution);
    assert_eq!(
        receipt.dependency,
        TextShapingFailureDependency::FontDatabase
    );
    assert_eq!(receipt.disposition, TextShapingFailureDisposition::Terminal);
    assert_eq!(receipt.source_range, None);
    assert_eq!(receipt.face, None);
}

#[test]
fn deferred_receipts_do_not_increment_terminal_diagnostics() {
    let mut diagnostics = TextShapingDiagnosticsReport::default();
    let failure = TextShapingFailure::font_generation_changed();

    diagnostics.record_deferred_failure(&failure);

    assert_eq!(diagnostics.failures.deferred_count, 1);
    assert_eq!(diagnostics.failures.terminal_count, 0);
    assert_eq!(diagnostics.backend_routes.deferred_run_count, 1);
    assert_eq!(diagnostics.backend_routes.terminal_run_count, 0);
}

#[test]
fn deferred_failure_merges_request_work_once() {
    let mut request = TextShapingRequestDiagnostics::EMPTY;
    request.shaping_attempt_count = 2;
    request.font_generation_restart_count = 2;
    request.font_resolution.resolution_request_count = 3;
    request.font_resolution.fallback_selection_count = 1;
    let failure = TextShapingFailure::font_generation_changed().with_request_diagnostics(request);
    let mut diagnostics = TextShapingDiagnosticsReport::default();

    diagnostics.record_deferred_failure(&failure);

    assert_eq!(diagnostics.shaping_attempt_count, 2);
    assert_eq!(diagnostics.font_generation_restart_count, 2);
    assert_eq!(diagnostics.font_resolution.resolution_request_count, 3);
    assert_eq!(diagnostics.font_resolution.fallback_selection_count, 1);
    assert_eq!(diagnostics.failures.deferred_count, 1);
}

#[test]
fn failure_report_counts_by_stable_code_without_allocating_labels() {
    let mut report = TextShapingFailureReport::default();
    let receipt = TextShapingFailureReceipt {
        code: TextShapingFailureCode::BackendGlyphInvalidClusterOffset,
        phase: TextShapingFailurePhase::BackendValidation,
        source_range: Some(TextRange { start: 1, end: 2 }),
        face: Some(FontFaceId(2)),
        dependency: TextShapingFailureDependency::ShapingBackend,
        disposition: TextShapingFailureDisposition::AlternateBackend,
        budget: None,
    };

    report.record(receipt);

    assert_eq!(report.observed_count, 1);
    assert_eq!(report.alternate_backend_count, 1);
    assert_eq!(report.terminal_count, 0);
    assert_eq!(report.count(receipt.code), 1);
    assert_eq!(report.last_failure, Some(receipt));
}

#[test]
fn stable_failure_code_labels_are_unique_and_low_cardinality() {
    let mut labels = TextShapingFailureCode::ALL.map(TextShapingFailureCode::as_str);
    labels.sort_unstable();

    assert!(labels.windows(2).all(|pair| pair[0] != pair[1]));
    assert!(labels.iter().all(|label| label.starts_with("text.")));
}
