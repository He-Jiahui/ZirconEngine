use super::TextLayoutFallbackReport;
use crate::core::framework::text::TextDirection;
use crate::core::framework::text::TextLayoutError;
use crate::text::cache::ShapedRunCacheLookupKey;
use crate::text::layout_session::{
    shape_request_outcome, GenerationTaggedShapedRun, SharedTextLayoutSession,
};
use crate::text::shaping::{TextShapingFailureCode, TextShapingOutcome};
use crate::text::{BackendShapeRequest, TextRange, TextStyle};

#[test]
fn generation_defer_does_not_count_as_a_fallback() {
    let mut report = TextLayoutFallbackReport::default();
    report.record(&TextLayoutError::FontGenerationChanged);

    assert_eq!(report.generation_deferred_count, 1);
    assert_eq!(report.fallback_count, 0);
}

#[test]
fn report_identifies_the_compiled_unicode_snapshot() {
    let report = TextLayoutFallbackReport::default();
    let snapshot = crate::text::compiled_unicode_data_snapshot_id();

    assert_eq!(report.unicode_data_generation, snapshot.generation());
    assert_eq!(report.unicode_data_fingerprint, snapshot.fingerprint());
}

#[test]
fn bidi_invariants_are_not_collapsed_into_other_errors() {
    let mut report = TextLayoutFallbackReport::default();
    report.record(&TextLayoutError::BidiInvariant);

    assert_eq!(report.fallback_count, 1);
    assert_eq!(report.bidi_invariant_count, 1);
    assert_eq!(report.other_error_count, 0);
}

#[test]
fn oversized_geometry_is_not_collapsed_into_other_errors() {
    let mut report = TextLayoutFallbackReport::default();
    report.record(&TextLayoutError::GeometryTooLarge);

    assert_eq!(report.fallback_count, 1);
    assert_eq!(report.geometry_too_large_count, 1);
    assert_eq!(report.other_error_count, 0);
}

#[test]
fn deferred_shaping_outcome_never_enters_the_session_cache() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        "generation changed",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 18 },
    );
    let lookup = ShapedRunCacheLookupKey::from_request(&request);
    let outcome = session.consume_shaping_outcome(
        &lookup,
        lookup.font_database_generation(),
        TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged),
    );

    assert!(matches!(
        outcome,
        TextShapingOutcome::Deferred(failure)
            if failure.error() == &TextLayoutError::FontGenerationChanged
                && failure.receipt().is_some_and(|receipt|
                    receipt.code == TextShapingFailureCode::FontGenerationChanged)
    ));
    assert!(session.shaped_runs.is_empty());
    assert_eq!(session.shaped_runs.report().insert_count, 0);
    assert_eq!(
        session.diagnostics_report().shaping.failures.deferred_count,
        1
    );
    assert_eq!(
        session
            .diagnostics_report()
            .shaping
            .backend_routes
            .terminal_run_count,
        0
    );
}

#[test]
fn ready_shaping_outcome_from_a_retired_generation_is_deferred() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        "retired generation",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 18 },
    );
    let lookup = ShapedRunCacheLookupKey::from_request(&request);
    let run = shape_request_outcome(request)
        .into_result()
        .expect("test input must shape at a stable generation");
    let outcome = session.consume_shaping_outcome(
        &lookup,
        lookup.font_database_generation(),
        TextShapingOutcome::Ready(GenerationTaggedShapedRun {
            run,
            font_generation: lookup.font_database_generation().saturating_sub(1),
            request_diagnostics: Default::default(),
        }),
    );

    assert!(matches!(
        outcome,
        TextShapingOutcome::Deferred(failure)
            if failure.error() == &TextLayoutError::FontGenerationChanged
                && failure.receipt().is_some_and(|receipt|
                    receipt.code == TextShapingFailureCode::FontGenerationChanged)
    ));
    assert!(session.shaped_runs.is_empty());
    assert_eq!(session.shaped_runs.report().insert_count, 0);
    assert_eq!(
        session.diagnostics_report().shaping.failures.deferred_count,
        1
    );
}
