use crate::core::framework::text::TextLayoutError;
use crate::text::model::TextShapingRequestDiagnostics;
use crate::text::shaping::{TextShapingDiagnosticsReport, TextShapingFailure};
use crate::text::{
    ShapedGlyphRun, TextLayoutGeometryOwner, TextLayoutGeometryViolation,
    compiled_unicode_data_snapshot_id,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextLayoutFallbackReport {
    pub unicode_data_generation: u64,
    pub unicode_data_fingerprint: u64,
    pub fallback_count: u64,
    pub generation_deferred_count: u64,
    pub invalid_font_size_count: u64,
    pub invalid_language_count: u64,
    pub bidi_invariant_count: u64,
    pub geometry_too_large_count: u64,
    pub other_error_count: u64,
}

impl Default for TextLayoutFallbackReport {
    fn default() -> Self {
        let unicode_data = compiled_unicode_data_snapshot_id();
        Self {
            unicode_data_generation: unicode_data.generation(),
            unicode_data_fingerprint: unicode_data.fingerprint(),
            fallback_count: 0,
            generation_deferred_count: 0,
            invalid_font_size_count: 0,
            invalid_language_count: 0,
            bidi_invariant_count: 0,
            geometry_too_large_count: 0,
            other_error_count: 0,
        }
    }
}

impl TextLayoutFallbackReport {
    pub(crate) fn record(&mut self, error: &TextLayoutError) {
        if matches!(error, TextLayoutError::FontGenerationChanged) {
            self.generation_deferred_count = self.generation_deferred_count.saturating_add(1);
            return;
        }
        self.fallback_count = self.fallback_count.saturating_add(1);
        match error {
            TextLayoutError::InvalidFontSize => {
                self.invalid_font_size_count = self.invalid_font_size_count.saturating_add(1);
            }
            TextLayoutError::InvalidLanguage => {
                self.invalid_language_count = self.invalid_language_count.saturating_add(1);
            }
            TextLayoutError::BidiInvariant => {
                self.bidi_invariant_count = self.bidi_invariant_count.saturating_add(1);
            }
            TextLayoutError::GeometryTooLarge => {
                self.geometry_too_large_count = self.geometry_too_large_count.saturating_add(1);
            }
            _ => {
                self.other_error_count = self.other_error_count.saturating_add(1);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryRejectionReceipt {
    pub(crate) owner: TextLayoutGeometryOwner,
    pub(crate) source_range: Option<(u32, u32)>,
    pub(crate) attempted_extent: f32,
    pub(crate) admitted_extent: f32,
    pub(crate) work_units: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextLayoutGeometryReport {
    pub(crate) rejection_count: u64,
    pub(crate) last_rejection: Option<TextLayoutGeometryRejectionReceipt>,
}

impl TextLayoutGeometryReport {
    fn record_rejection(
        &mut self,
        owner: TextLayoutGeometryOwner,
        violation: TextLayoutGeometryViolation,
        source_range: Option<(u32, u32)>,
        work_units: usize,
    ) {
        self.rejection_count = self.rejection_count.saturating_add(1);
        self.last_rejection = Some(TextLayoutGeometryRejectionReceipt {
            owner,
            source_range,
            attempted_extent: violation.attempted_extent,
            admitted_extent: violation.admitted_extent,
            work_units,
        });
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextLayoutSessionDiagnostics {
    pub(crate) layout_fallbacks: TextLayoutFallbackReport,
    pub(crate) shaping: TextShapingDiagnosticsReport,
    pub(crate) geometry: TextLayoutGeometryReport,
}

impl TextLayoutSessionDiagnostics {
    pub(crate) fn record_layout_error(&mut self, error: &TextLayoutError) {
        self.layout_fallbacks.record(error);
    }

    pub(crate) fn record_geometry_rejection(
        &mut self,
        owner: TextLayoutGeometryOwner,
        violation: TextLayoutGeometryViolation,
        source_range: Option<(u32, u32)>,
        work_units: usize,
    ) {
        self.geometry
            .record_rejection(owner, violation, source_range, work_units);
    }

    pub(crate) fn record_ready_run(
        &mut self,
        run: &ShapedGlyphRun,
        request: TextShapingRequestDiagnostics,
    ) {
        self.shaping.record_ready_run(run, request);
    }

    pub(crate) fn record_terminal_failure(&mut self, failure: &TextShapingFailure) {
        self.shaping.record_terminal_failure(failure);
    }

    pub(crate) fn record_deferred_failure(&mut self, failure: &TextShapingFailure) {
        self.shaping.record_deferred_failure(failure);
    }

    pub(crate) fn merge_shaping(&mut self, report: TextShapingDiagnosticsReport) {
        self.shaping.merge(report);
    }
}

#[cfg(test)]
mod tests {
    use super::TextLayoutFallbackReport;
    use crate::core::framework::text::TextDirection;
    use crate::core::framework::text::TextLayoutError;
    use crate::text::cache::ShapedRunCacheLookupKey;
    use crate::text::layout_session::{
        GenerationTaggedShapedRun, SharedTextLayoutSession, shape_request_outcome,
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
}
