use crate::core::framework::text::TextLayoutError;
use crate::text::model::TextShapingRequestDiagnostics;
use crate::text::shaping::{TextShapingDiagnosticsReport, TextShapingFailure};
use crate::text::{
    compiled_unicode_data_snapshot_id, ShapedGlyphRun, TextLayoutGeometryOwner,
    TextLayoutGeometryViolation,
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
        // 字体 generation 变化单独计为 deferred，避免与已经选择 fallback 的终态错误混计。
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
#[path = "tests/diagnostics.rs"]
mod tests;
