use serde::{Deserialize, Serialize};

use super::TextRange;
use super::font::FontFaceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[non_exhaustive]
pub enum TextShapingFailureCode {
    ItemizationInvalidSourceRange = 0,
    ItemizationMissingFallbackSpan = 1,
    BidiInvariant = 2,
    BackendFontDatabase = 3,
    FontSourceBudgetExceeded = 4,
    BackendFaceParse = 5,
    BackendEmptyGlyphOutput = 6,
    DirectInvalidSourceRange = 7,
    BackendGlyphEmptyOutput = 8,
    BackendGlyphInvalidClusterOffset = 9,
    BackendGlyphNonFiniteMetrics = 10,
    BackendGlyphNonMonotonicClusterOrder = 11,
    FontPrimaryUnavailable = 12,
    FontGenerationChanged = 13,
}

impl TextShapingFailureCode {
    pub const COUNT: usize = 14;
    pub const ALL: [Self; Self::COUNT] = [
        Self::ItemizationInvalidSourceRange,
        Self::ItemizationMissingFallbackSpan,
        Self::BidiInvariant,
        Self::BackendFontDatabase,
        Self::FontSourceBudgetExceeded,
        Self::BackendFaceParse,
        Self::BackendEmptyGlyphOutput,
        Self::DirectInvalidSourceRange,
        Self::BackendGlyphEmptyOutput,
        Self::BackendGlyphInvalidClusterOffset,
        Self::BackendGlyphNonFiniteMetrics,
        Self::BackendGlyphNonMonotonicClusterOrder,
        Self::FontPrimaryUnavailable,
        Self::FontGenerationChanged,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ItemizationInvalidSourceRange => "text.itemization.invalid_source_range",
            Self::ItemizationMissingFallbackSpan => "text.itemization.missing_fallback_span",
            Self::BidiInvariant => "text.bidi.invariant",
            Self::BackendFontDatabase => "text.backend.font_database",
            Self::FontSourceBudgetExceeded => "text.budget.font_source_admission",
            Self::BackendFaceParse => "text.backend.face_parse",
            Self::BackendEmptyGlyphOutput => "text.backend.empty_glyph_output",
            Self::DirectInvalidSourceRange => "text.direct.invalid_source_range",
            Self::BackendGlyphEmptyOutput => "text.backend_glyph.empty_output",
            Self::BackendGlyphInvalidClusterOffset => "text.backend_glyph.invalid_cluster_offset",
            Self::BackendGlyphNonFiniteMetrics => "text.backend_glyph.non_finite_metrics",
            Self::BackendGlyphNonMonotonicClusterOrder => {
                "text.backend_glyph.non_monotonic_cluster_order"
            }
            Self::FontPrimaryUnavailable => "text.font.primary_unavailable",
            Self::FontGenerationChanged => "text.font.generation_changed",
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextShapingFailurePhase {
    InputValidation,
    Itemization,
    BidiAnalysis,
    FontResolution,
    FontLoad,
    BackendShape,
    BackendValidation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextShapingFailureDependency {
    SourceText,
    UnicodeBidiData,
    FontDatabase,
    FontFace,
    ShapingBackend,
    WorkBudget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextShapingFailureDisposition {
    Terminal,
    AlternateBackend,
    Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextShapingBudgetKind {
    FontSourceAdmission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TextShapingFailureReceipt {
    pub code: TextShapingFailureCode,
    pub phase: TextShapingFailurePhase,
    pub source_range: Option<TextRange>,
    pub face: Option<FontFaceId>,
    pub dependency: TextShapingFailureDependency,
    pub disposition: TextShapingFailureDisposition,
    pub budget: Option<TextShapingBudgetKind>,
}

impl TextShapingFailureReceipt {
    pub const fn allows_alternate_backend(self) -> bool {
        matches!(
            self.disposition,
            TextShapingFailureDisposition::AlternateBackend
        )
    }
}

/// Fixed-cardinality work receipt for one or more font-resolution attempts.
///
/// This report records only work executed by the current shaping request. A generation-owned
/// resolution-cache hit therefore increments the hit and selection counters without replaying the
/// historical candidate visits that originally produced the cached value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextFontResolutionReport {
    pub(crate) primary_text_request_count: u64,
    pub(crate) primary_text_fast_path_count: u64,
    pub(crate) resolution_request_count: u64,
    pub(crate) resolution_cache_hit_count: u64,
    pub(crate) resolution_cache_miss_count: u64,
    pub(crate) candidate_cache_hit_count: u64,
    pub(crate) candidate_cache_miss_count: u64,
    pub(crate) decision_coverage_call_count: u64,
    pub(crate) primary_coverage_rejection_count: u64,
    pub(crate) complete_candidate_visit_count: u64,
    pub(crate) complete_candidate_rejection_count: u64,
    pub(crate) partial_candidate_visit_count: u64,
    pub(crate) primary_selection_count: u64,
    pub(crate) fallback_selection_count: u64,
    pub(crate) partial_coverage_selection_count: u64,
    pub(crate) last_resort_selection_count: u64,
    pub(crate) depth_limit_selection_count: u64,
}

impl TextFontResolutionReport {
    pub(crate) fn merge(&mut self, other: Self) {
        self.primary_text_request_count = self
            .primary_text_request_count
            .saturating_add(other.primary_text_request_count);
        self.primary_text_fast_path_count = self
            .primary_text_fast_path_count
            .saturating_add(other.primary_text_fast_path_count);
        self.resolution_request_count = self
            .resolution_request_count
            .saturating_add(other.resolution_request_count);
        self.resolution_cache_hit_count = self
            .resolution_cache_hit_count
            .saturating_add(other.resolution_cache_hit_count);
        self.resolution_cache_miss_count = self
            .resolution_cache_miss_count
            .saturating_add(other.resolution_cache_miss_count);
        self.candidate_cache_hit_count = self
            .candidate_cache_hit_count
            .saturating_add(other.candidate_cache_hit_count);
        self.candidate_cache_miss_count = self
            .candidate_cache_miss_count
            .saturating_add(other.candidate_cache_miss_count);
        self.decision_coverage_call_count = self
            .decision_coverage_call_count
            .saturating_add(other.decision_coverage_call_count);
        self.primary_coverage_rejection_count = self
            .primary_coverage_rejection_count
            .saturating_add(other.primary_coverage_rejection_count);
        self.complete_candidate_visit_count = self
            .complete_candidate_visit_count
            .saturating_add(other.complete_candidate_visit_count);
        self.complete_candidate_rejection_count = self
            .complete_candidate_rejection_count
            .saturating_add(other.complete_candidate_rejection_count);
        self.partial_candidate_visit_count = self
            .partial_candidate_visit_count
            .saturating_add(other.partial_candidate_visit_count);
        self.primary_selection_count = self
            .primary_selection_count
            .saturating_add(other.primary_selection_count);
        self.fallback_selection_count = self
            .fallback_selection_count
            .saturating_add(other.fallback_selection_count);
        self.partial_coverage_selection_count = self
            .partial_coverage_selection_count
            .saturating_add(other.partial_coverage_selection_count);
        self.last_resort_selection_count = self
            .last_resort_selection_count
            .saturating_add(other.last_resort_selection_count);
        self.depth_limit_selection_count = self
            .depth_limit_selection_count
            .saturating_add(other.depth_limit_selection_count);
    }
}

/// Request-owned shaping diagnostics carried beside, never inside, the cacheable glyph artifact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingRequestDiagnostics {
    pub(crate) shaping_attempt_count: u64,
    pub(crate) font_generation_restart_count: u64,
    pub(crate) font_resolution: TextFontResolutionReport,
}

impl TextShapingRequestDiagnostics {
    pub(crate) const EMPTY: Self = Self {
        shaping_attempt_count: 0,
        font_generation_restart_count: 0,
        font_resolution: TextFontResolutionReport {
            primary_text_request_count: 0,
            primary_text_fast_path_count: 0,
            resolution_request_count: 0,
            resolution_cache_hit_count: 0,
            resolution_cache_miss_count: 0,
            candidate_cache_hit_count: 0,
            candidate_cache_miss_count: 0,
            decision_coverage_call_count: 0,
            primary_coverage_rejection_count: 0,
            complete_candidate_visit_count: 0,
            complete_candidate_rejection_count: 0,
            partial_candidate_visit_count: 0,
            primary_selection_count: 0,
            fallback_selection_count: 0,
            partial_coverage_selection_count: 0,
            last_resort_selection_count: 0,
            depth_limit_selection_count: 0,
        },
    };

    pub(crate) fn merge(&mut self, other: Self) {
        self.shaping_attempt_count = self
            .shaping_attempt_count
            .saturating_add(other.shaping_attempt_count);
        self.font_generation_restart_count = self
            .font_generation_restart_count
            .saturating_add(other.font_generation_restart_count);
        self.font_resolution.merge(other.font_resolution);
    }
}

/// Provenance retained when a horizontal run uses an alternate backend.
///
/// Empty `alternate_ranges` means the alternate backend owns the complete run. Non-empty ranges
/// identify the source spans composed into otherwise direct output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextHorizontalCompositionReceipt {
    pub alternate_ranges: Vec<TextRange>,
    pub first_failure: TextShapingFailureReceipt,
}
