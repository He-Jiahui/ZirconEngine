use std::ops::Range;

use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::text::font::{font_query_for_text_style, FallbackResolution, FontDatabase};
use crate::text::model::TextFontResolutionReport;
use crate::text::{BackendShapeRequest, FontFaceId, InstancedFaceId, TextRange};

use super::script_segment::ParagraphTextAnalysis;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FallbackTextSpan {
    pub(crate) range: Range<usize>,
    pub(crate) family: Option<String>,
    /// The query's resolved primary face, retained even when this span shapes with fallback.
    ///
    /// Text03 uses this identity for the eventual composite-line metric policy. It must not be
    /// inferred from the first selected span because a fallback glyph can start the source.
    pub(crate) primary_face: FontFaceId,
    pub(crate) resolution: FallbackResolution,
    pub(crate) instance: Option<InstancedFaceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FallbackItemizationError {
    PrimaryFaceUnavailable,
}

/// Returns the resolver-selected primary face that is common to one itemized request.
///
/// The selected face of the first span may be a fallback, so callers that form a canonical
/// fragment must use this identity rather than inspect an arbitrary selected span.
pub(crate) fn fallback_primary_face(spans: &[FallbackTextSpan]) -> Option<FontFaceId> {
    let primary_face = spans.first()?.primary_face;
    debug_assert!(
        spans.iter().all(|span| span.primary_face == primary_face),
        "one fallback itemization request must retain exactly one primary face"
    );
    Some(primary_face)
}

pub(crate) fn fallback_text_spans(
    text: &str,
    request: BackendShapeRequest<'_>,
    database: &FontDatabase,
    analysis: &ParagraphTextAnalysis,
) -> Result<Vec<FallbackTextSpan>, FallbackItemizationError> {
    fallback_text_spans_with_report(text, request, database, analysis).map(|(spans, _)| spans)
}

pub(crate) fn fallback_text_spans_with_report(
    text: &str,
    request: BackendShapeRequest<'_>,
    database: &FontDatabase,
    analysis: &ParagraphTextAnalysis,
) -> Result<(Vec<FallbackTextSpan>, TextFontResolutionReport), FallbackItemizationError> {
    debug_assert_eq!(
        analysis.unicode_data_snapshot(),
        request.unicode_data_snapshot(),
        "fallback analysis must use the request-bound Unicode snapshot"
    );
    let query = font_query_for_text_style(request.style);
    let query =
        database.constrain_font_query_to_request_owner(&query, request.style.font.as_deref());
    let default_family = request
        .style
        .font_family
        .as_deref()
        .map(str::trim)
        .filter(|family| !family.is_empty());
    let mut face_resolver = database
        .begin_shaping_face_resolution_for_request(
            query.as_ref(),
            request.style.font.as_deref(),
            request.language_fallback_key(),
        )
        .ok_or(FallbackItemizationError::PrimaryFaceUnavailable)?;
    let primary_face = face_resolver.primary_face();
    if face_resolver.primary_covers_text(text) {
        let face = primary_face;
        let family = database
            .face_family_name(face)
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        let instance = database
            .effective_instance_id(
                face,
                TextStyle::normalized_font_weight(request.style.font_weight),
            )
            .ok();
        let spans = vec![FallbackTextSpan {
            range: 0..text.len(),
            family,
            primary_face,
            resolution: face_resolver.primary_resolution(),
            instance,
        }];
        let report = face_resolver.take_resolution_report();
        return Ok((spans, report));
    }
    let mut spans = Vec::<FallbackTextSpan>::new();
    let mut cluster_codepoints = Vec::new();
    for (start, cluster) in text.grapheme_indices(true) {
        let end = start + cluster.len();
        let range = TextRange { start, end };
        cluster_codepoints.clear();
        cluster_codepoints.extend(cluster.chars());
        let resolution =
            face_resolver.resolve(analysis.font_script_for_range(range), &cluster_codepoints);
        let face = resolution.face();
        let instance = database
            .effective_instance_id(
                face,
                TextStyle::normalized_font_weight(request.style.font_weight),
            )
            .ok();
        if let Some(previous) = spans.last_mut() {
            if previous.resolution == resolution
                && previous.instance == instance
                && previous.range.end == start
            {
                previous.range.end = end;
                continue;
            }
        }
        let family = database
            .face_family_name(face)
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        spans.push(FallbackTextSpan {
            range: start..end,
            family,
            primary_face,
            resolution,
            instance,
        });
    }
    let report = face_resolver.take_resolution_report();
    Ok((spans, report))
}

#[cfg(test)]
#[path = "tests/fallback_spans.rs"]
mod tests;
