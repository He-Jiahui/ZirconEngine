use crate::text::font::FontDatabaseError;
use crate::text::model::{
    TextFontResolutionReport, TextShapingBudgetKind, TextShapingFailureCode,
    TextShapingFailureDependency, TextShapingFailureDisposition, TextShapingFailurePhase,
    TextShapingFailureReceipt, TextShapingRequestDiagnostics,
};
use crate::text::{ShapedGlyphRun, TextOrientation, TextRange, compiled_unicode_data_snapshot_id};

use super::backend_error::{BackendFontOperation, BackendShapeError};
use super::bidi::BidiInvariantError;
use super::direct_error::{BackendGlyphInvariantKind, DirectShapeError};
use super::fallback_spans::FallbackItemizationError;
use super::itemize::ItemizationError;
use super::outcome::TextShapingFailure;

impl TextShapingFailureReceipt {
    pub(crate) const fn font_generation_changed() -> Self {
        Self {
            code: TextShapingFailureCode::FontGenerationChanged,
            phase: TextShapingFailurePhase::FontResolution,
            source_range: None,
            face: None,
            dependency: TextShapingFailureDependency::FontDatabase,
            disposition: TextShapingFailureDisposition::Deferred,
            budget: None,
        }
    }

    const fn from_fallback_itemization(error: FallbackItemizationError) -> Self {
        match error {
            FallbackItemizationError::PrimaryFaceUnavailable => Self {
                code: TextShapingFailureCode::FontPrimaryUnavailable,
                phase: TextShapingFailurePhase::FontResolution,
                source_range: None,
                face: None,
                dependency: TextShapingFailureDependency::FontDatabase,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
        }
    }

    fn from_direct(error: &DirectShapeError, orientation: TextOrientation) -> Self {
        let alternate_backend = |mut receipt: Self| {
            if orientation == TextOrientation::Horizontal {
                receipt.disposition = TextShapingFailureDisposition::AlternateBackend;
            }
            receipt
        };
        match error {
            DirectShapeError::Itemization(ItemizationError::InvalidSourceRange { range }) => Self {
                code: TextShapingFailureCode::ItemizationInvalidSourceRange,
                phase: TextShapingFailurePhase::Itemization,
                source_range: Some(*range),
                face: None,
                dependency: TextShapingFailureDependency::SourceText,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
            DirectShapeError::Itemization(ItemizationError::MissingFallbackSpan { range }) => {
                Self {
                    code: TextShapingFailureCode::ItemizationMissingFallbackSpan,
                    phase: TextShapingFailurePhase::Itemization,
                    source_range: Some(*range),
                    face: None,
                    dependency: TextShapingFailureDependency::FontDatabase,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }
            }
            DirectShapeError::Itemization(ItemizationError::BidiInvariant(source))
            | DirectShapeError::BidiInvariant(source) => bidi_receipt(source),
            DirectShapeError::Backend { range, source } => match source {
                BackendShapeError::FontDatabase {
                    operation,
                    face,
                    source: FontDatabaseError::SourceBudget { .. },
                } => Self {
                    code: TextShapingFailureCode::FontSourceBudgetExceeded,
                    phase: backend_font_phase(*operation),
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::WorkBudget,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: Some(TextShapingBudgetKind::FontSourceAdmission),
                },
                BackendShapeError::FontDatabase {
                    operation, face, ..
                } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendFontDatabase,
                    phase: backend_font_phase(*operation),
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::FontDatabase,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
                BackendShapeError::FaceParseFailed { face, .. } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendFaceParse,
                    phase: TextShapingFailurePhase::FontLoad,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::FontFace,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
                BackendShapeError::EmptyGlyphOutput { face } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendEmptyGlyphOutput,
                    phase: TextShapingFailurePhase::BackendShape,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::ShapingBackend,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
            },
            DirectShapeError::InvalidSourceRange { range } => Self {
                code: TextShapingFailureCode::DirectInvalidSourceRange,
                phase: TextShapingFailurePhase::InputValidation,
                source_range: Some(*range),
                face: None,
                dependency: TextShapingFailureDependency::SourceText,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
            DirectShapeError::BackendGlyphInvariant { face, range, kind } => {
                alternate_backend(Self {
                    code: backend_glyph_code(*kind),
                    phase: TextShapingFailurePhase::BackendValidation,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::ShapingBackend,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                })
            }
        }
    }
}

impl TextShapingFailure {
    pub(crate) const fn font_generation_changed() -> Self {
        Self::with_receipt(
            crate::core::framework::text::TextLayoutError::FontGenerationChanged,
            TextShapingFailureReceipt::font_generation_changed(),
        )
    }
}

impl From<FallbackItemizationError> for TextShapingFailure {
    fn from(error: FallbackItemizationError) -> Self {
        Self::with_receipt(
            crate::core::framework::text::TextLayoutError::FontUnavailable,
            TextShapingFailureReceipt::from_fallback_itemization(error),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextShapingFailureReport {
    pub unicode_data_generation: u64,
    pub unicode_data_fingerprint: u64,
    pub observed_count: u64,
    pub alternate_backend_count: u64,
    pub deferred_count: u64,
    pub terminal_count: u64,
    counts: [u64; TextShapingFailureCode::COUNT],
    pub last_failure: Option<TextShapingFailureReceipt>,
}

impl TextShapingFailureReport {
    pub const fn count(self, code: TextShapingFailureCode) -> u64 {
        self.counts[code.index()]
    }

    pub(crate) fn record(&mut self, receipt: TextShapingFailureReceipt) {
        self.observed_count = self.observed_count.saturating_add(1);
        self.counts[receipt.code.index()] = self.counts[receipt.code.index()].saturating_add(1);
        match receipt.disposition {
            TextShapingFailureDisposition::AlternateBackend => {
                self.alternate_backend_count = self.alternate_backend_count.saturating_add(1);
            }
            TextShapingFailureDisposition::Deferred => {
                self.deferred_count = self.deferred_count.saturating_add(1);
            }
            TextShapingFailureDisposition::Terminal => {
                self.terminal_count = self.terminal_count.saturating_add(1);
            }
        }
        self.last_failure = Some(receipt);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.observed_count = self.observed_count.saturating_add(other.observed_count);
        self.alternate_backend_count = self
            .alternate_backend_count
            .saturating_add(other.alternate_backend_count);
        self.deferred_count = self.deferred_count.saturating_add(other.deferred_count);
        self.terminal_count = self.terminal_count.saturating_add(other.terminal_count);
        for (count, other_count) in self.counts.iter_mut().zip(other.counts) {
            *count = count.saturating_add(other_count);
        }
        if other.last_failure.is_some() {
            self.last_failure = other.last_failure;
        }
    }
}

impl Default for TextShapingFailureReport {
    fn default() -> Self {
        let unicode_data = compiled_unicode_data_snapshot_id();
        Self {
            unicode_data_generation: unicode_data.generation(),
            unicode_data_fingerprint: unicode_data.fingerprint(),
            observed_count: 0,
            alternate_backend_count: 0,
            deferred_count: 0,
            terminal_count: 0,
            counts: [0; TextShapingFailureCode::COUNT],
            last_failure: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingBackendRouteReport {
    pub(crate) direct_run_count: u64,
    pub(crate) alternate_run_count: u64,
    pub(crate) hybrid_run_count: u64,
    pub(crate) recovered_source_range_count: u64,
    pub(crate) deferred_run_count: u64,
    pub(crate) terminal_run_count: u64,
}

impl TextShapingBackendRouteReport {
    fn record_ready_run(&mut self, run: &ShapedGlyphRun) {
        let Some(receipt) = run.horizontal_composition_receipt.as_deref() else {
            self.direct_run_count = self.direct_run_count.saturating_add(1);
            return;
        };

        if receipt.alternate_ranges.is_empty() {
            self.alternate_run_count = self.alternate_run_count.saturating_add(1);
            self.recovered_source_range_count = self.recovered_source_range_count.saturating_add(1);
        } else {
            self.hybrid_run_count = self.hybrid_run_count.saturating_add(1);
            self.recovered_source_range_count = self
                .recovered_source_range_count
                .saturating_add(receipt.alternate_ranges.len() as u64);
        }
    }

    fn merge(&mut self, other: Self) {
        self.direct_run_count = self.direct_run_count.saturating_add(other.direct_run_count);
        self.alternate_run_count = self
            .alternate_run_count
            .saturating_add(other.alternate_run_count);
        self.hybrid_run_count = self.hybrid_run_count.saturating_add(other.hybrid_run_count);
        self.recovered_source_range_count = self
            .recovered_source_range_count
            .saturating_add(other.recovered_source_range_count);
        self.deferred_run_count = self
            .deferred_run_count
            .saturating_add(other.deferred_run_count);
        self.terminal_run_count = self
            .terminal_run_count
            .saturating_add(other.terminal_run_count);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingDiagnosticsReport {
    pub(crate) failures: TextShapingFailureReport,
    pub(crate) backend_routes: TextShapingBackendRouteReport,
    pub(crate) shaping_attempt_count: u64,
    pub(crate) font_generation_restart_count: u64,
    pub(crate) font_resolution: TextFontResolutionReport,
}

impl TextShapingDiagnosticsReport {
    pub(crate) fn record_ready_run(
        &mut self,
        run: &ShapedGlyphRun,
        request: TextShapingRequestDiagnostics,
    ) {
        self.record_request(request);
        if let Some(receipt) = run.horizontal_composition_receipt.as_deref() {
            self.failures.record(receipt.first_failure);
        }
        self.backend_routes.record_ready_run(run);
    }

    pub(crate) fn record_terminal_failure(&mut self, failure: &TextShapingFailure) {
        self.record_request(failure.request_diagnostics());
        let Some(receipt) = failure.receipt() else {
            return;
        };
        self.failures.record(receipt);
        self.backend_routes.terminal_run_count =
            self.backend_routes.terminal_run_count.saturating_add(1);
    }

    pub(crate) fn record_deferred_failure(&mut self, failure: &TextShapingFailure) {
        self.record_request(failure.request_diagnostics());
        let Some(receipt) = failure.receipt() else {
            return;
        };
        self.failures.record(receipt);
        self.backend_routes.deferred_run_count =
            self.backend_routes.deferred_run_count.saturating_add(1);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.failures.merge(other.failures);
        self.backend_routes.merge(other.backend_routes);
        self.shaping_attempt_count = self
            .shaping_attempt_count
            .saturating_add(other.shaping_attempt_count);
        self.font_generation_restart_count = self
            .font_generation_restart_count
            .saturating_add(other.font_generation_restart_count);
        self.font_resolution.merge(other.font_resolution);
    }

    fn record_request(&mut self, request: TextShapingRequestDiagnostics) {
        self.shaping_attempt_count = self
            .shaping_attempt_count
            .saturating_add(request.shaping_attempt_count);
        self.font_generation_restart_count = self
            .font_generation_restart_count
            .saturating_add(request.font_generation_restart_count);
        self.font_resolution.merge(request.font_resolution);
    }
}

pub(super) fn classify_direct_shape_failure(
    error: &DirectShapeError,
    orientation: TextOrientation,
) -> TextShapingFailureReceipt {
    TextShapingFailureReceipt::from_direct(error, orientation)
}

const fn backend_font_phase(operation: BackendFontOperation) -> TextShapingFailurePhase {
    match operation {
        BackendFontOperation::ResolveVariations => TextShapingFailurePhase::FontResolution,
        BackendFontOperation::LoadFaceBytes | BackendFontOperation::ResolveFaceIndex => {
            TextShapingFailurePhase::FontLoad
        }
    }
}

const fn backend_glyph_code(kind: BackendGlyphInvariantKind) -> TextShapingFailureCode {
    match kind {
        BackendGlyphInvariantKind::EmptyOutput => TextShapingFailureCode::BackendGlyphEmptyOutput,
        BackendGlyphInvariantKind::InvalidClusterOffset => {
            TextShapingFailureCode::BackendGlyphInvalidClusterOffset
        }
        BackendGlyphInvariantKind::NonFiniteMetrics => {
            TextShapingFailureCode::BackendGlyphNonFiniteMetrics
        }
        BackendGlyphInvariantKind::NonMonotonicClusterOrder => {
            TextShapingFailureCode::BackendGlyphNonMonotonicClusterOrder
        }
    }
}

fn bidi_receipt(source: &BidiInvariantError) -> TextShapingFailureReceipt {
    TextShapingFailureReceipt {
        code: TextShapingFailureCode::BidiInvariant,
        phase: TextShapingFailurePhase::BidiAnalysis,
        source_range: bidi_error_range(source),
        face: None,
        dependency: TextShapingFailureDependency::UnicodeBidiData,
        disposition: TextShapingFailureDisposition::Terminal,
        budget: None,
    }
}

fn bidi_error_range(error: &BidiInvariantError) -> Option<TextRange> {
    match *error {
        BidiInvariantError::InvalidResolvedRange { start, end }
        | BidiInvariantError::InvalidLineRange { start, end }
        | BidiInvariantError::LineOutsideParagraph { start, end }
        | BidiInvariantError::GlyphOutsideLine { start, end, .. } => Some(TextRange { start, end }),
        BidiInvariantError::MissingResolvedLevel { offset, .. }
        | BidiInvariantError::MissingResolvedRangeLevel { offset }
        | BidiInvariantError::MissingSignatureScalar { offset } => Some(TextRange {
            start: offset,
            end: offset,
        }),
        BidiInvariantError::NonMonotonicGlyphRange { start, .. } => {
            Some(TextRange { start, end: start })
        }
        BidiInvariantError::ProjectionCardinalityMismatch { .. }
        | BidiInvariantError::AdvanceCardinalityMismatch { .. }
        | BidiInvariantError::MissingLogicalCluster { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::asset::assets::FontSourceBudgetError;
    use crate::core::framework::text::TextLayoutError;
    use crate::text::shaping::fallback_spans::FallbackItemizationError;

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

        let horizontal =
            TextShapingFailureReceipt::from_direct(&error, TextOrientation::Horizontal);
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
        let bidi =
            DirectShapeError::from(BidiInvariantError::InvalidLineRange { start: 9, end: 4 });
        let bidi_receipt =
            TextShapingFailureReceipt::from_direct(&bidi, TextOrientation::Horizontal);
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
        let failure =
            TextShapingFailure::font_generation_changed().with_request_diagnostics(request);
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
}
