use crate::text::font::{FontDatabase, SelectedFaceLineEnvelope, SelectedFaceLineExtents};
use crate::text::layout_geometry::finite_sum;
use crate::text::{
    HorizontalGlyphMetricSpan, ShapedGlyph, ShapedGlyphRun, ShapedHardLine,
    TextHorizontalCompositionReceipt, TextRange, TextShapingFailureReceipt,
};

use super::position_glyphs;
use crate::text::shaping::direct_error::DirectShapeError;

pub(super) struct HorizontalDirectHole {
    pub(super) range: TextRange,
    pub(super) error: DirectShapeError,
}

pub(in crate::text::shaping) struct HorizontalPartialShape {
    pub(super) direct: ShapedGlyphRun,
    pub(super) holes: Vec<HorizontalDirectHole>,
}

pub(in crate::text::shaping) enum HorizontalDirectShapeAttempt {
    Complete(ShapedGlyphRun),
    Partial(HorizontalPartialShape),
}

#[derive(Debug)]
pub(in crate::text::shaping) struct HorizontalCompositionSuccess {
    pub(in crate::text::shaping) shaped: ShapedGlyphRun,
    pub(in crate::text::shaping) alternate_glyph_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum HorizontalCompositionError {
    IncompatibleRunIdentity,
    IncompatibleLineTopology,
    InvalidHoleRange,
    NonMonotonicGlyphOrder,
    DirectGlyphOverlapsHole,
    AlternateGlyphCrossesHole,
    MissingAlternateGlyph,
}

impl HorizontalDirectShapeAttempt {
    pub(super) fn from_parts(direct: ShapedGlyphRun, holes: Vec<HorizontalDirectHole>) -> Self {
        debug_assert!(holes.windows(2).all(|holes| {
            holes[0].range.start < holes[0].range.end && holes[0].range.end <= holes[1].range.start
        }));
        debug_assert!(holes.last().is_none_or(|hole| {
            hole.range.start < hole.range.end
                && hole.range.end
                    <= direct
                        .source_range
                        .end
                        .saturating_sub(direct.source_range.start)
        }));
        if holes.is_empty() {
            Self::Complete(direct)
        } else {
            Self::Partial(HorizontalPartialShape { direct, holes })
        }
    }
}

impl HorizontalPartialShape {
    pub(in crate::text::shaping) fn errors(&self) -> impl Iterator<Item = &DirectShapeError> {
        self.holes.iter().map(|hole| &hole.error)
    }

    pub(in crate::text::shaping) fn hole_count(&self) -> usize {
        self.holes.len()
    }

    pub(in crate::text::shaping) fn direct_glyph_count(&self) -> usize {
        self.direct.lines.iter().map(|line| line.glyphs.len()).sum()
    }
}

/// 用 alternate glyph 仅填补 direct holes。先验证 source/run/line 身份和单调范围，再重排 glyph、重算 metrics，并写入 composition provenance。
pub(in crate::text::shaping) fn compose_horizontal_partial(
    partial: HorizontalPartialShape,
    alternate: ShapedGlyphRun,
    database: &FontDatabase,
    font_size: f32,
    requested_line_height: f32,
    first_failure: TextShapingFailureReceipt,
) -> Result<HorizontalCompositionSuccess, (HorizontalCompositionError, ShapedGlyphRun)> {
    if let Err(error) = validate_run_identity(&partial.direct, &alternate) {
        return Err((error, alternate));
    }
    // 成功组合把 hole 与 provenance 回执转换到 owner 坐标，与 glyph 的绝对范围一起发布。
    let absolute_holes = match absolute_holes(&partial.direct, &partial.holes) {
        Ok(holes) => holes,
        Err(error) => return Err((error, alternate)),
    };
    if let Err(error) = qualify_candidate(&partial.direct, &alternate, &absolute_holes) {
        return Err((error, alternate));
    }

    let HorizontalPartialShape { mut direct, holes } = partial;
    debug_assert_eq!(holes.len(), absolute_holes.len());
    let mut alternate_lines = alternate.lines.into_iter();
    let mut metric_spans = Vec::new();
    let mut raw_metrics = Vec::with_capacity(direct.lines.len());
    let mut hole_cursor = 0_usize;
    let mut alternate_glyph_count = 0_usize;

    for (line_index, direct_line) in direct.lines.iter_mut().enumerate() {
        let alternate_line = alternate_lines
            .next()
            .expect("qualified alternate line topology");
        let direct_ascent = direct_line.baseline;
        let direct_descent = (direct_line.line_height - direct_line.baseline).max(0.0);
        let alternate_ascent = alternate_line.baseline;
        let alternate_descent = (alternate_line.line_height - alternate_line.baseline).max(0.0);
        let mut selected =
            select_alternate_glyphs(alternate_line.glyphs, &absolute_holes, &mut hole_cursor);
        alternate_glyph_count = alternate_glyph_count.saturating_add(selected.len());
        direct_line.glyphs.append(&mut selected);
        direct_line
            .glyphs
            .sort_by_key(|glyph| (glyph.source_range.start, glyph.source_range.end));
        rebuild_line(
            direct_line,
            line_index,
            direct.primary_face_id,
            database,
            font_size,
            requested_line_height,
            direct_ascent.max(alternate_ascent),
            direct_descent.max(alternate_descent),
            &mut raw_metrics,
            &mut metric_spans,
        );
    }
    debug_assert!(alternate_lines.next().is_none());

    direct.measured_width = direct
        .lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    direct.measured_height = finite_sum(direct.lines.iter().map(|line| line.line_height));
    let mut artifact_failure = first_failure;
    // Request diagnostics use request-local ranges; shaped artifacts use owner source coordinates.
    artifact_failure.source_range = absolute_holes.first().copied();
    direct.horizontal_composition_receipt = Some(Box::new(TextHorizontalCompositionReceipt {
        alternate_ranges: absolute_holes,
        first_failure: artifact_failure,
    }));
    direct.horizontal_line_raw_metrics = raw_metrics;
    direct.horizontal_glyph_metric_spans = metric_spans;
    Ok(HorizontalCompositionSuccess {
        shaped: direct,
        alternate_glyph_count,
    })
}

/// 线性游标同时验证 direct 不覆盖 hole、alternate 不跨 hole 且每个 hole 至少有一个候选 glyph。
fn qualify_candidate(
    direct: &ShapedGlyphRun,
    alternate: &ShapedGlyphRun,
    holes: &[TextRange],
) -> Result<(), HorizontalCompositionError> {
    let mut alternate_counts = vec![0_usize; holes.len()];
    let mut direct_hole_cursor = 0_usize;
    let mut alternate_hole_cursor = 0_usize;
    let mut previous_direct_range = None;
    let mut previous_alternate_range = None;
    for (direct_line, alternate_line) in direct.lines.iter().zip(&alternate.lines) {
        validate_line_identity(direct_line, alternate_line)?;
        validate_direct_glyphs(
            &direct_line.glyphs,
            holes,
            &mut direct_hole_cursor,
            &mut previous_direct_range,
        )?;
        for glyph in &alternate_line.glyphs {
            validate_monotonic_glyph(glyph, &mut previous_alternate_range)?;
            while holes
                .get(alternate_hole_cursor)
                .is_some_and(|hole| hole.end <= glyph.source_range.start)
            {
                alternate_hole_cursor += 1;
            }
            let Some(hole) = holes.get(alternate_hole_cursor).copied() else {
                continue;
            };
            if !ranges_overlap(glyph.source_range, hole) {
                continue;
            }
            if glyph.source_range.start < hole.start || glyph.source_range.end > hole.end {
                return Err(HorizontalCompositionError::AlternateGlyphCrossesHole);
            }
            alternate_counts[alternate_hole_cursor] =
                alternate_counts[alternate_hole_cursor].saturating_add(1);
        }
    }
    if alternate_counts.contains(&0) {
        Err(HorizontalCompositionError::MissingAlternateGlyph)
    } else {
        Ok(())
    }
}

fn validate_run_identity(
    direct: &ShapedGlyphRun,
    alternate: &ShapedGlyphRun,
) -> Result<(), HorizontalCompositionError> {
    ((std::sync::Arc::ptr_eq(&direct.source_text, &alternate.source_text)
        || direct.source_text == alternate.source_text)
        && direct.source_range == alternate.source_range
        && direct.unicode_data_snapshot == alternate.unicode_data_snapshot
        && direct.direction == alternate.direction
        && direct.orientation == alternate.orientation
        && direct.vertical_mode == alternate.vertical_mode
        && direct.include_kerning == alternate.include_kerning
        && direct.lines.len() == alternate.lines.len())
    .then_some(())
    .ok_or(HorizontalCompositionError::IncompatibleRunIdentity)
}

fn validate_line_identity(
    direct: &ShapedHardLine,
    alternate: &ShapedHardLine,
) -> Result<(), HorizontalCompositionError> {
    (direct.line_index == alternate.line_index
        && direct.source_range == alternate.source_range
        && direct.visual_range == alternate.visual_range)
        .then_some(())
        .ok_or(HorizontalCompositionError::IncompatibleLineTopology)
}

fn absolute_holes(
    direct: &ShapedGlyphRun,
    holes: &[HorizontalDirectHole],
) -> Result<Vec<TextRange>, HorizontalCompositionError> {
    let source_len = direct
        .source_range
        .end
        .checked_sub(direct.source_range.start)
        .ok_or(HorizontalCompositionError::InvalidHoleRange)?;
    let mut absolute = Vec::with_capacity(holes.len());
    for hole in holes {
        if hole.range.start >= hole.range.end || hole.range.end > source_len {
            return Err(HorizontalCompositionError::InvalidHoleRange);
        }
        let range = TextRange {
            start: direct
                .source_range
                .start
                .checked_add(hole.range.start)
                .ok_or(HorizontalCompositionError::InvalidHoleRange)?,
            end: direct
                .source_range
                .start
                .checked_add(hole.range.end)
                .ok_or(HorizontalCompositionError::InvalidHoleRange)?,
        };
        if absolute
            .last()
            .is_some_and(|previous: &TextRange| previous.end > range.start)
        {
            return Err(HorizontalCompositionError::InvalidHoleRange);
        }
        absolute.push(range);
    }
    Ok(absolute)
}

fn validate_direct_glyphs(
    glyphs: &[ShapedGlyph],
    holes: &[TextRange],
    hole_cursor: &mut usize,
    previous_range: &mut Option<TextRange>,
) -> Result<(), HorizontalCompositionError> {
    for glyph in glyphs {
        validate_monotonic_glyph(glyph, previous_range)?;
        while holes
            .get(*hole_cursor)
            .is_some_and(|hole| hole.end <= glyph.source_range.start)
        {
            *hole_cursor += 1;
        }
        if holes
            .get(*hole_cursor)
            .is_some_and(|hole| ranges_overlap(glyph.source_range, *hole))
        {
            return Err(HorizontalCompositionError::DirectGlyphOverlapsHole);
        }
    }
    Ok(())
}

fn validate_monotonic_glyph(
    glyph: &ShapedGlyph,
    previous_range: &mut Option<TextRange>,
) -> Result<(), HorizontalCompositionError> {
    let range = glyph.source_range;
    if range.start > range.end
        || previous_range
            .is_some_and(|previous| (previous.start, previous.end) > (range.start, range.end))
    {
        return Err(HorizontalCompositionError::NonMonotonicGlyphOrder);
    }
    *previous_range = Some(range);
    Ok(())
}

fn select_alternate_glyphs(
    glyphs: Vec<ShapedGlyph>,
    holes: &[TextRange],
    hole_cursor: &mut usize,
) -> Vec<ShapedGlyph> {
    let mut selected = Vec::new();
    for glyph in glyphs {
        while holes
            .get(*hole_cursor)
            .is_some_and(|hole| hole.end <= glyph.source_range.start)
        {
            *hole_cursor += 1;
        }
        let Some(hole) = holes.get(*hole_cursor).copied() else {
            continue;
        };
        if !ranges_overlap(glyph.source_range, hole) {
            continue;
        }
        if glyph.source_range.start >= hole.start && glyph.source_range.end <= hole.end {
            selected.push(glyph);
        }
    }
    selected
}

const fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start < right.end && right.start < left.end
}

#[allow(clippy::too_many_arguments)]
fn rebuild_line(
    line: &mut ShapedHardLine,
    line_index: usize,
    primary_face: Option<crate::text::FontFaceId>,
    database: &FontDatabase,
    font_size: f32,
    requested_line_height: f32,
    fallback_ascent: f32,
    fallback_descent: f32,
    raw_metrics: &mut Vec<Option<crate::text::HorizontalLineRawMetrics>>,
    metric_spans: &mut Vec<HorizontalGlyphMetricSpan>,
) {
    let mut extents = SelectedFaceLineExtents::default();
    if let Some(primary_face) = primary_face {
        extents.include_primary_face(database, primary_face, font_size);
    }
    let mut glyph_start = 0_usize;
    while glyph_start < line.glyphs.len() {
        let face = line.glyphs[glyph_start].font_id;
        let glyph_end =
            glyph_start + line.glyphs[glyph_start..].partition_point(|glyph| glyph.font_id == face);
        if let Some(metrics) = face.and_then(|face| extents.include_face(database, face, font_size))
        {
            metric_spans.push(HorizontalGlyphMetricSpan {
                line_index,
                glyph_start,
                glyph_end,
                metrics,
            });
        }
        glyph_start = glyph_end;
    }
    let fallback_envelope = SelectedFaceLineEnvelope {
        baseline_from_top: fallback_ascent,
        line_height: requested_line_height.max(fallback_ascent + fallback_descent),
    };
    let envelope = extents
        .resolve_content_envelope(requested_line_height)
        .unwrap_or(fallback_envelope);
    let cursor = position_glyphs(&mut line.glyphs);
    line.measured_width = cursor;
    line.baseline = envelope.baseline_from_top;
    line.line_height = envelope.line_height;
    raw_metrics.push(extents.raw_horizontal_metrics());
}

#[cfg(test)]
#[path = "tests/composition.rs"]
mod tests;
