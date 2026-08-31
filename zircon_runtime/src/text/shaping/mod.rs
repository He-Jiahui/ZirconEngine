//! Shared text shaping owner. Third-party text backend types stay in leaf modules.

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
mod analysis_profile;
mod backend_error;
mod bidi;
mod cosmic;
mod direct_error;
mod emoji_presentation;
mod failure_receipt;
mod fallback_spans;
mod horizontal;
mod itemize;
mod line_break;
mod normalize;
mod outcome;
mod script_segment;
mod source_profile;
mod vertical;
mod work_budget;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::text::VerticalMode;
use crate::text::font::{FontCollectionSnapshot, shared_font_collection_snapshot};
use crate::text::{BackendShapeRequest, ShapedGlyphRun};
use crate::text::{TextRange, TextStyle};

pub use super::model::{
    TextShapingBudgetKind, TextShapingFailureCode, TextShapingFailureDependency,
    TextShapingFailureDisposition, TextShapingFailurePhase, TextShapingFailureReceipt,
};
pub(crate) use bidi::{
    BidiInvariantError, BidiLineOrder, BidiLineSignature, analyze_bidi_line,
    capture_bidi_line_signature, mirrored_bidi_char, resolve_bidi_base_direction,
};
pub use failure_receipt::TextShapingFailureReport;
pub(crate) use failure_receipt::{TextShapingBackendRouteReport, TextShapingDiagnosticsReport};
pub(crate) use fallback_spans::{
    FallbackItemizationError, FallbackTextSpan, fallback_primary_face, fallback_text_spans,
    fallback_text_spans_with_report,
};
pub(crate) use outcome::{
    TextLayoutOutcome, TextShapingCompletion, TextShapingFailure, TextShapingOutcome,
};
pub(crate) use script_segment::ParagraphTextAnalysis;
pub(crate) use vertical::{vertical_glyph_advance, vertical_glyph_rotation};
pub(crate) use work_budget::{TextShapingWorkBudget, TextShapingWorkReport};

pub(crate) fn shape_text(
    request: BackendShapeRequest<'_>,
) -> Result<ShapedGlyphRun, TextShapingFailure> {
    shape_text_with_diagnostics(request).map(|completion| completion.into_parts().0)
}

pub(crate) fn shape_text_with_diagnostics(
    request: BackendShapeRequest<'_>,
) -> Result<TextShapingCompletion<ShapedGlyphRun>, TextShapingFailure> {
    let font_collection = shared_font_collection_snapshot();
    shape_text_with_diagnostics_in_font_collection(request, &font_collection)
}

pub(crate) fn shape_text_with_diagnostics_in_font_collection(
    request: BackendShapeRequest<'_>,
    font_collection: &FontCollectionSnapshot,
) -> Result<TextShapingCompletion<ShapedGlyphRun>, TextShapingFailure> {
    let canonical_request = request.canonicalized().map_err(TextShapingFailure::from)?;
    let request = canonical_request.request();
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    analysis_profile::begin(request.text.len());
    let result = cosmic::shape_text_in_font_collection(request, font_collection);
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    analysis_profile::finish();
    result
}

pub(crate) trait TextShapeRunProvider {
    fn font_collection_revision(&self) -> crate::text::font::FontCollectionRevision {
        crate::text::font::shared_font_collection_service().revision()
    }

    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome;

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        let _ = vertical_mode;
        self.shape_horizontal_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}

pub(crate) struct VerticalTextShapeRunProvider<'a, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    provider: &'a mut P,
    vertical_mode: VerticalMode,
}

impl<'a, P> VerticalTextShapeRunProvider<'a, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    pub(crate) fn new(provider: &'a mut P, vertical_mode: VerticalMode) -> Self {
        Self {
            provider,
            vertical_mode,
        }
    }
}

impl<P> TextShapeRunProvider for VerticalTextShapeRunProvider<'_, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    fn font_collection_revision(&self) -> crate::text::font::FontCollectionRevision {
        self.provider.font_collection_revision()
    }

    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.provider.shape_vertical_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            self.vertical_mode,
            include_kerning,
        )
    }

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.provider.shape_vertical_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        )
    }
}

/// Compatibility provider for one-shot/editor calls.
///
/// The snapshot is captured when the provider is created so a multi-line operation cannot mix
/// font collections when a publication happens between shape requests. Retained Runtime paths
/// should pass their `SharedTextLayoutSession` or an explicit collection-bound provider instead.
#[derive(Clone)]
pub(crate) struct DirectTextShapeRunProvider {
    font_collection: FontCollectionSnapshot,
}

impl Default for DirectTextShapeRunProvider {
    fn default() -> Self {
        Self {
            font_collection: shared_font_collection_snapshot(),
        }
    }
}

impl DirectTextShapeRunProvider {
    #[cfg(test)]
    pub(crate) fn from_font_collection(font_collection: FontCollectionSnapshot) -> Self {
        Self { font_collection }
    }
}

impl TextShapeRunProvider for DirectTextShapeRunProvider {
    fn font_collection_revision(&self) -> crate::text::font::FontCollectionRevision {
        self.font_collection.revision()
    }

    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        let request = BackendShapeRequest::horizontal_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        );
        TextShapingOutcome::from_shape_result(
            shape_text_with_diagnostics_in_font_collection(request, &self.font_collection)
                .map(|completion| completion.into_parts().0),
        )
        .map(Arc::new)
    }

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        let request = BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        );
        TextShapingOutcome::from_shape_result(
            shape_text_with_diagnostics_in_font_collection(request, &self.font_collection)
                .map(|completion| completion.into_parts().0),
        )
        .map(Arc::new)
    }
}

pub(crate) struct FontCollectionTextShapeRunProvider<'a> {
    font_collection: &'a FontCollectionSnapshot,
}

impl<'a> FontCollectionTextShapeRunProvider<'a> {
    pub(crate) const fn new(font_collection: &'a FontCollectionSnapshot) -> Self {
        Self { font_collection }
    }

    fn shape_request(&self, request: BackendShapeRequest<'_>) -> TextShapingOutcome {
        TextShapingOutcome::from_shape_result(
            shape_text_with_diagnostics_in_font_collection(request, self.font_collection)
                .map(|completion| completion.into_parts().0),
        )
        .map(Arc::new)
    }
}

impl TextShapeRunProvider for FontCollectionTextShapeRunProvider<'_> {
    fn font_collection_revision(&self) -> crate::text::font::FontCollectionRevision {
        self.font_collection.revision()
    }

    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.shape_request(BackendShapeRequest::horizontal_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        ))
    }

    fn shape_vertical_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> TextShapingOutcome {
        self.shape_request(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        ))
    }
}

#[cfg(test)]
pub(crate) fn shape_horizontal_range(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
) -> ShapedGlyphRun {
    shape_horizontal_range_with_kerning(text, style, direction, source_range, true)
}

#[cfg(test)]
pub(crate) fn shape_horizontal_range_with_kerning(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    let request = BackendShapeRequest::horizontal_with_kerning(
        text,
        style,
        direction,
        source_range,
        include_kerning,
    );
    shape_text(request).expect("test shaping request must be valid")
}
