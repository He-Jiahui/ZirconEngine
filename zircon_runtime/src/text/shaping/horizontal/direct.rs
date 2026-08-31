use crate::text::font::{FontDatabase, SelectedFaceLineEnvelope, SelectedFaceLineExtents};
use crate::text::{
    BackendShapeRequest, HorizontalGlyphMetricSpan, ShapedGlyph, ShapedGlyphRun, ShapedHardLine,
    TextRange,
};

use super::backend::{HorizontalBackendRun, shape_horizontal_run};
use super::composition::{HorizontalDirectHole, HorizontalDirectShapeAttempt};
use crate::text::shaping::bidi::BidiParagraph;
use crate::text::shaping::cosmic::{cluster_flags, resolved_line_height};
use crate::text::shaping::direct_error::{
    BackendGlyphInvariantKind, DirectShapeError, validate_backend_glyphs,
};
use crate::text::shaping::fallback_spans::{FallbackTextSpan, fallback_primary_face};
use crate::text::shaping::itemize::{
    LogicalSegment, logical_segments_for_line, restore_backend_cluster_logical_order,
    virtual_hard_break_glyph,
};
use crate::text::shaping::line_break::LineBreakOpportunityMap;
use crate::text::shaping::script_segment::ParagraphTextAnalysis;

pub(in crate::text::shaping) fn shape_horizontal_request(
    request: BackendShapeRequest<'_>,
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[FallbackTextSpan],
    analysis: &ParagraphTextAnalysis,
    database: &FontDatabase,
) -> Result<HorizontalDirectShapeAttempt, DirectShapeError> {
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
    let line_height = resolved_line_height(request);
    let mut lines = Vec::new();
    let mut horizontal_line_raw_metrics = Vec::new();
    let mut horizontal_glyph_metric_spans = Vec::new();
    let mut holes = Vec::new();

    for (line_index, hard_line) in crate::text::hard_lines(request.text)
        .into_iter()
        .enumerate()
    {
        let line_range = hard_line.content.clone();
        let mut glyphs = Vec::new();
        let mut selected_face_extents = SelectedFaceLineExtents::default();
        if let Some(primary_face) = fallback_primary_face(fallback_spans) {
            selected_face_extents.include_primary_face(
                database,
                primary_face,
                request.style.font_size,
            );
        }
        let segments = logical_segments_for_line(
            request.text,
            line_range.clone(),
            fallback_spans,
            analysis,
            bidi,
            None,
        )?;
        for segment in segments {
            let face = segment.face;
            let span_metrics =
                selected_face_extents.include_face(database, face, request.style.font_size);
            let segment_glyphs =
                match shape_segment(request, line_range.start, segment, &line_breaks, database) {
                    Ok(glyphs) => glyphs,
                    Err(error) => {
                        holes.push(HorizontalDirectHole {
                            range: segment.range,
                            error,
                        });
                        continue;
                    }
                };
            let glyph_start = glyphs.len();
            glyphs.extend(segment_glyphs);
            if glyph_start < glyphs.len() {
                if let Some(metrics) = span_metrics {
                    horizontal_glyph_metric_spans.push(HorizontalGlyphMetricSpan {
                        line_index,
                        glyph_start,
                        glyph_end: glyphs.len(),
                        metrics,
                    });
                }
            }
        }
        if let Some(separator) = virtual_hard_break_glyph(request, &hard_line, bidi, analysis)? {
            glyphs.push(separator);
        }

        let mut cursor = 0.0_f32;
        for glyph in &mut glyphs {
            glyph.x = cursor;
            cursor += glyph.advance.max(0.0);
        }
        let selected_face_envelope = selected_face_extents
            .resolve_content_envelope(line_height)
            .unwrap_or(SelectedFaceLineEnvelope {
                baseline_from_top: request.style.font_size.max(1.0) * 0.8,
                line_height,
            });
        horizontal_line_raw_metrics.push(selected_face_extents.raw_horizontal_metrics());
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
            measured_width: cursor,
            baseline: selected_face_envelope.baseline_from_top,
            line_height: selected_face_envelope.line_height,
            glyphs,
        });
    }

    let measured_width = lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    let direct = ShapedGlyphRun {
        source_text: crate::text::shaping::source_profile::materialize_source_text(request),
        source_range: request.source_range,
        unicode_data_snapshot: request.unicode_data_snapshot(),
        primary_face_id: fallback_primary_face(fallback_spans),
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width,
        measured_height: lines.iter().map(|line| line.line_height).sum::<f32>(),
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics,
        horizontal_glyph_metric_spans,
        lines,
    };
    Ok(HorizontalDirectShapeAttempt::from_parts(direct, holes))
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
    let mut backend = shape_horizontal_run(
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
    .map_err(|source| DirectShapeError::backend(segment.range, source))?;
    valid_backend_run(&backend, text).map_err(|kind| {
        DirectShapeError::backend_glyph_invariant(segment.face, segment.range, kind)
    })?;
    restore_backend_cluster_logical_order(&mut backend.glyphs, segment.direction, |glyph| {
        glyph.source_offset
    })
    .ok_or_else(|| {
        DirectShapeError::backend_glyph_invariant(
            segment.face,
            segment.range,
            BackendGlyphInvariantKind::NonMonotonicClusterOrder,
        )
    })?;
    let mut glyphs = Vec::with_capacity(backend.glyphs.len());
    let mut backend_start = 0;
    while backend_start < backend.glyphs.len() {
        let source_offset = backend.glyphs[backend_start].source_offset;
        let backend_end = backend_start
            + backend.glyphs[backend_start..]
                .partition_point(|glyph| glyph.source_offset == source_offset);
        let cluster_end = backend
            .glyphs
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
        let unsafe_to_break = backend.glyphs[backend_start..backend_end]
            .iter()
            .any(|glyph| glyph.unsafe_to_break);
        for (cluster_glyph_index, backend_glyph) in backend.glyphs[backend_start..backend_end]
            .iter()
            .copied()
            .enumerate()
        {
            let cluster_start = cluster_glyph_index == 0;
            let flags = cluster_flags(
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
                advance: backend_glyph.advance.abs(),
                x: 0.0,
                y: 0.0,
                offset_x: backend_glyph.x_offset,
                offset_y: -backend_glyph.y_offset,
                direction: segment.direction,
                bidi_level: segment.bidi_level,
                cluster_flags: flags,
                rotation: crate::text::ShapedGlyphRotation::None,
                script: segment.script,
            });
        }
        backend_start = backend_end;
    }
    Ok(glyphs)
}

fn valid_backend_run(
    run: &HorizontalBackendRun,
    text: &str,
) -> Result<(), BackendGlyphInvariantKind> {
    validate_backend_glyphs(
        &run.glyphs,
        text,
        |glyph| glyph.source_offset,
        |glyph| {
            glyph.advance.is_finite() && glyph.x_offset.is_finite() && glyph.y_offset.is_finite()
        },
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::text::font::{FontDatabase, SelectedFaceLineExtents};
    use crate::text::shaping::direct_error::BackendGlyphInvariantKind;

    use crate::text::shaping::horizontal::backend::{HorizontalBackendGlyph, HorizontalBackendRun};

    use super::valid_backend_run;

    #[test]
    fn direct_backend_validation_reports_a_non_boundary_cluster_offset() {
        let run = HorizontalBackendRun {
            glyphs: vec![HorizontalBackendGlyph {
                glyph_id: 1,
                source_offset: 1,
                unsafe_to_break: false,
                advance: 1.0,
                x_offset: 0.0,
                y_offset: 0.0,
            }],
        };

        assert_eq!(
            valid_backend_run(&run, "é"),
            Err(BackendGlyphInvariantKind::InvalidClusterOffset)
        );
    }

    #[test]
    fn direct_line_uses_scaled_selected_face_content_envelope() {
        let mut database = FontDatabase::default();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
        let face = database
            .register_font_file(source, Some("Direct Metrics Face"), 0)
            .expect("register tracked font");
        let source_metrics = database
            .face_metrics(face)
            .expect("face metrics query")
            .expect("tracked face metrics");
        let mut extents = SelectedFaceLineExtents::default();
        let _ = extents.include_face(&database, face, 20.0);
        let envelope = extents
            .resolve_content_envelope(24.0)
            .expect("face metrics");
        let expected_ascent = f32::from(source_metrics.ascender.max(0)) * 20.0
            / f32::from(source_metrics.units_per_em);

        assert!(envelope.baseline_from_top >= expected_ascent);
        assert!(envelope.line_height >= 24.0);
        assert!((envelope.baseline_from_top - 16.0).abs() > 0.01);
    }
}
