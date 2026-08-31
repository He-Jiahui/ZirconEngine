use crate::core::framework::text::TextDirection;
use crate::text::font::FontDatabase;
use crate::text::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun, ShapedHardLine,
    TextRange, TextVerticalGlyphDecisionBasis, TextVerticalGlyphFallbackReason,
    TextVerticalGlyphFeatureSet, TextVerticalGlyphOrientation, TextVerticalGlyphSubstitution,
    VerticalMode,
};

use super::backend::{VerticalBackendDirection, shape_vertical_run};
use super::orientation::{
    VerticalShapeOrientation, transform_or_rotate_rotation, vertical_glyph_metrics_for_rotation,
};
use crate::text::shaping::bidi::BidiParagraph;
use crate::text::shaping::cosmic::cluster_flags;
use crate::text::shaping::direct_error::{
    BackendGlyphInvariantKind, DirectShapeError, validate_backend_glyphs,
};
use crate::text::shaping::fallback_spans::{FallbackTextSpan, fallback_primary_face};
use crate::text::shaping::horizontal::{HorizontalBackendRun, shape_horizontal_run};
use crate::text::shaping::itemize::{
    LogicalSegment, logical_segments_for_line, restore_backend_cluster_logical_order,
    virtual_hard_break_glyph,
};
use crate::text::shaping::line_break::LineBreakOpportunityMap;
use crate::text::shaping::script_segment::ParagraphTextAnalysis;

#[derive(Clone, Copy)]
struct BackendGlyph {
    glyph_id: u32,
    source_offset: usize,
    unsafe_to_break: bool,
    advance: f32,
    offset_x: f32,
    offset_y: f32,
    vertical_substituted: bool,
}

pub(in crate::text::shaping) fn shape_vertical_request(
    request: BackendShapeRequest<'_>,
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[FallbackTextSpan],
    analysis: &ParagraphTextAnalysis,
    database: &FontDatabase,
) -> Result<ShapedGlyphRun, DirectShapeError> {
    debug_assert_eq!(
        bidi.unicode_data_snapshot(),
        request.unicode_data_snapshot(),
        "Bidi analysis must use the request-bound Unicode snapshot"
    );
    debug_assert_eq!(
        analysis.unicode_data_snapshot(),
        request.unicode_data_snapshot(),
        "script analysis must use the request-bound Unicode snapshot"
    );
    let line_breaks =
        LineBreakOpportunityMap::for_snapshot(request.text, request.unicode_data_snapshot());
    debug_assert_eq!(
        line_breaks.unicode_data_snapshot(),
        request.unicode_data_snapshot(),
        "line-break analysis must use the request-bound Unicode snapshot"
    );
    let column_width = request.style.font_size.max(1.0);
    let mut lines = Vec::new();
    let mut populated_columns = 0_usize;
    let mut measured_height = 0.0_f32;

    for (line_index, hard_line) in crate::text::hard_lines(request.text)
        .into_iter()
        .enumerate()
    {
        let line_range = hard_line.content.clone();
        let mut glyphs = Vec::new();
        let segments = logical_segments_for_line(
            request.text,
            line_range.clone(),
            fallback_spans,
            analysis,
            bidi,
            Some(request.vertical_mode),
        )?;
        for segment in segments {
            let segment_glyphs =
                shape_segment(request, line_range.start, segment, &line_breaks, database)?;
            glyphs.extend(segment_glyphs);
        }
        if let Some(mut separator) = virtual_hard_break_glyph(request, &hard_line, bidi, analysis)?
        {
            separator.cluster_flags =
                separator
                    .cluster_flags
                    .with_vertical_decision(TextVerticalGlyphDecisionBasis {
                        orientation: TextVerticalGlyphOrientation::Upright,
                        features: TextVerticalGlyphFeatureSet::None,
                        substitution: TextVerticalGlyphSubstitution::NotChecked,
                        fallback_reason: TextVerticalGlyphFallbackReason::NonRenderingControl,
                    });
            glyphs.push(separator);
        }

        let column_height = position_vertical_glyphs(request, &mut glyphs, column_width);
        if !glyphs.is_empty() {
            populated_columns += 1;
        }
        measured_height = measured_height.max(column_height);
        let full_range = hard_line.source_range();
        lines.push(ShapedHardLine {
            line_index,
            source_range: TextRange {
                start: request.source_range.start + full_range.start,
                end: request.source_range.start + full_range.end,
            },
            visual_range: TextRange {
                start: 0,
                end: full_range.end.saturating_sub(full_range.start),
            },
            measured_width: column_height,
            baseline: column_width * 0.5,
            line_height: column_width,
            glyphs,
        });
    }

    Ok(ShapedGlyphRun {
        source_text: crate::text::shaping::source_profile::materialize_source_text(request),
        source_range: request.source_range,
        unicode_data_snapshot: request.unicode_data_snapshot(),
        primary_face_id: fallback_primary_face(fallback_spans),
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: populated_columns as f32 * column_width,
        measured_height,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines,
    })
}

fn shape_segment(
    request: BackendShapeRequest<'_>,
    line_start: usize,
    segment: LogicalSegment,
    line_breaks: &LineBreakOpportunityMap,
    database: &FontDatabase,
) -> Result<Vec<ShapedGlyph>, DirectShapeError> {
    let text = request
        .text
        .get(segment.range.start..segment.range.end)
        .ok_or(DirectShapeError::InvalidSourceRange {
            range: segment.range,
        })?;
    let (mut backend, vertical_features) = if !matches!(
        segment.vertical_orientation,
        VerticalShapeOrientation::Sideways
    ) {
        let shaped = shape_vertical_run(
            database,
            segment.face,
            segment.instance,
            text,
            vertical_backend_direction(segment.direction),
            segment.script.iso15924,
            request.language,
            request.features(),
            request.include_kerning,
            matches!(
                segment.vertical_orientation,
                VerticalShapeOrientation::TransformOrRotate
            ),
            crate::text::TextStyle::normalized_font_weight(request.style.font_weight),
            request.style.font_size,
        )
        .map_err(|source| DirectShapeError::backend(segment.range, source))?;
        (
            shaped
                .glyphs
                .into_iter()
                .map(|glyph| BackendGlyph {
                    glyph_id: glyph.glyph_id,
                    source_offset: glyph.source_offset,
                    unsafe_to_break: glyph.unsafe_to_break,
                    advance: glyph.y_advance.abs(),
                    offset_x: glyph.x_offset,
                    offset_y: -glyph.y_offset,
                    vertical_substituted: glyph.vertical_substituted,
                })
                .collect::<Vec<_>>(),
            shaped.features,
        )
    } else {
        (
            horizontal_backend_glyphs(
                shape_horizontal_run(
                    database,
                    segment.face,
                    segment.instance,
                    text,
                    segment.direction,
                    segment.script.iso15924,
                    request.language,
                    request.features(),
                    request.include_kerning,
                    crate::text::TextStyle::normalized_font_weight(request.style.font_weight),
                    request.style.font_size,
                )
                .map_err(|source| DirectShapeError::backend(segment.range, source))?,
            ),
            TextVerticalGlyphFeatureSet::None,
        )
    };
    valid_backend_glyphs(&backend, text).map_err(|kind| {
        DirectShapeError::backend_glyph_invariant(segment.face, segment.range, kind)
    })?;
    restore_backend_cluster_logical_order(&mut backend, segment.direction, |glyph| {
        glyph.source_offset
    })
    .ok_or_else(|| {
        DirectShapeError::backend_glyph_invariant(
            segment.face,
            segment.range,
            BackendGlyphInvariantKind::NonMonotonicClusterOrder,
        )
    })?;

    let mut glyphs = Vec::with_capacity(backend.len());
    let mut backend_start = 0;
    while backend_start < backend.len() {
        let source_offset = backend[backend_start].source_offset;
        let backend_end = backend_start
            + backend[backend_start..]
                .partition_point(|glyph| glyph.source_offset == source_offset);
        let cluster_end = backend
            .get(backend_end)
            .map(|glyph| glyph.source_offset)
            .unwrap_or(text.len());
        let local_range = TextRange {
            start: segment.range.start + source_offset,
            end: segment.range.start + cluster_end,
        };
        let cluster_text = request
            .text
            .get(local_range.start..local_range.end)
            .ok_or(DirectShapeError::InvalidSourceRange { range: local_range })?;
        let decision = resolved_cluster_decision(
            segment,
            request.vertical_mode,
            vertical_features,
            &backend[backend_start..backend_end],
        );
        let unsafe_to_break = backend[backend_start..backend_end]
            .iter()
            .any(|glyph| glyph.unsafe_to_break);
        for (cluster_glyph_index, backend_glyph) in backend[backend_start..backend_end]
            .iter()
            .copied()
            .enumerate()
        {
            let cluster_start = cluster_glyph_index == 0;
            let mut flags = cluster_flags(
                cluster_text,
                segment.direction,
                cluster_start,
                if cluster_start {
                    line_breaks.flags_for_cluster(local_range.start, local_range.end)
                } else {
                    Default::default()
                },
            )
            .with_direct_break_safety(unsafe_to_break);
            if cluster_start {
                flags = flags.with_vertical_decision(decision);
            }
            glyphs.push(ShapedGlyph {
                glyph_id: backend_glyph.glyph_id,
                font_id: Some(segment.face),
                font_instance_id: segment.instance,
                source_range: TextRange {
                    start: request.source_range.start + local_range.start,
                    end: request.source_range.start + local_range.end,
                },
                visual_range: TextRange {
                    start: local_range.start.saturating_sub(line_start),
                    end: local_range.end.saturating_sub(line_start),
                },
                advance: backend_glyph.advance,
                x: 0.0,
                y: 0.0,
                offset_x: backend_glyph.offset_x,
                offset_y: backend_glyph.offset_y,
                direction: segment.direction,
                bidi_level: segment.bidi_level,
                cluster_flags: flags,
                rotation: decision_rotation(decision),
                script: segment.script,
            });
        }
        backend_start = backend_end;
    }
    Ok(glyphs)
}

fn resolved_cluster_decision(
    segment: LogicalSegment,
    vertical_mode: VerticalMode,
    features: TextVerticalGlyphFeatureSet,
    glyphs: &[BackendGlyph],
) -> TextVerticalGlyphDecisionBasis {
    let (orientation, features, substitution, fallback_reason) = match segment.vertical_orientation
    {
        VerticalShapeOrientation::Upright => (
            TextVerticalGlyphOrientation::Upright,
            features,
            TextVerticalGlyphSubstitution::NotChecked,
            TextVerticalGlyphFallbackReason::None,
        ),
        VerticalShapeOrientation::Sideways => (
            TextVerticalGlyphOrientation::Sideways,
            TextVerticalGlyphFeatureSet::None,
            TextVerticalGlyphSubstitution::NotChecked,
            if matches!(vertical_mode, VerticalMode::Sideways) {
                TextVerticalGlyphFallbackReason::ForcedSideways
            } else {
                TextVerticalGlyphFallbackReason::UnicodeSideways
            },
        ),
        VerticalShapeOrientation::TransformOrRotate => {
            let substituted = glyphs.iter().any(|glyph| glyph.vertical_substituted);
            (
                TextVerticalGlyphOrientation::TransformOrRotate,
                features,
                if substituted {
                    TextVerticalGlyphSubstitution::Observed
                } else {
                    TextVerticalGlyphSubstitution::NotObserved
                },
                if substituted {
                    TextVerticalGlyphFallbackReason::None
                } else {
                    TextVerticalGlyphFallbackReason::NoVerticalSubstitution
                },
            )
        }
    };
    TextVerticalGlyphDecisionBasis {
        orientation,
        features,
        substitution,
        fallback_reason,
    }
}

const fn decision_rotation(decision: TextVerticalGlyphDecisionBasis) -> ShapedGlyphRotation {
    match decision.orientation {
        TextVerticalGlyphOrientation::Upright => ShapedGlyphRotation::None,
        TextVerticalGlyphOrientation::Sideways => ShapedGlyphRotation::Cw90,
        TextVerticalGlyphOrientation::TransformOrRotate => transform_or_rotate_rotation(matches!(
            decision.substitution,
            TextVerticalGlyphSubstitution::Observed
        )),
    }
}

fn horizontal_backend_glyphs(run: HorizontalBackendRun) -> Vec<BackendGlyph> {
    run.glyphs
        .into_iter()
        .map(|glyph| BackendGlyph {
            glyph_id: glyph.glyph_id,
            source_offset: glyph.source_offset,
            unsafe_to_break: glyph.unsafe_to_break,
            advance: glyph.advance.abs(),
            offset_x: glyph.x_offset,
            offset_y: -glyph.y_offset,
            vertical_substituted: false,
        })
        .collect()
}

fn valid_backend_glyphs(
    glyphs: &[BackendGlyph],
    text: &str,
) -> Result<(), BackendGlyphInvariantKind> {
    validate_backend_glyphs(
        glyphs,
        text,
        |glyph| glyph.source_offset,
        |glyph| {
            glyph.advance.is_finite() && glyph.offset_x.is_finite() && glyph.offset_y.is_finite()
        },
    )
}

fn position_vertical_glyphs(
    request: BackendShapeRequest<'_>,
    glyphs: &mut [ShapedGlyph],
    column_width: f32,
) -> f32 {
    let mut cursor_y = 0.0_f32;
    let mut glyph_start = 0_usize;
    while glyph_start < glyphs.len() {
        let source_range = glyphs[glyph_start].source_range;
        let mut glyph_end = glyph_start + 1;
        while glyph_end < glyphs.len() && glyphs[glyph_end].source_range == source_range {
            glyph_end += 1;
        }

        let cluster_text = super::source_cluster_text(request, source_range);
        let shaped_advance = glyphs[glyph_start..glyph_end]
            .iter()
            .map(|glyph| glyph.advance.max(0.0))
            .sum::<f32>();
        let native_vertical_advance =
            matches!(glyphs[glyph_start].rotation, ShapedGlyphRotation::None)
                .then_some(shaped_advance);
        let metrics = vertical_glyph_metrics_for_rotation(
            cluster_text,
            glyphs[glyph_start].rotation,
            shaped_advance,
            column_width,
            native_vertical_advance,
        );
        for (cluster_glyph_index, glyph) in glyphs[glyph_start..glyph_end].iter_mut().enumerate() {
            glyph.x = column_width * 0.5;
            glyph.y = cursor_y;
            glyph.offset_x += metrics.offset_x;
            glyph.rotation = metrics.rotation;
            glyph.advance = if cluster_glyph_index == 0 {
                metrics.advance
            } else {
                0.0
            };
        }
        cursor_y += metrics.advance;
        glyph_start = glyph_end;
    }
    cursor_y
}

pub(super) fn vertical_backend_direction(direction: TextDirection) -> VerticalBackendDirection {
    if matches!(direction, TextDirection::RightToLeft) {
        VerticalBackendDirection::BottomToTop
    } else {
        VerticalBackendDirection::TopToBottom
    }
}
