use crate::text::font::FontDatabase;
use crate::text::{
    BackendShapeRequest, FontFaceId, ShapedGlyph, ShapedGlyphRun, ShapedTextLine, TextRange,
};

use super::backend::{HorizontalBackendRun, shape_horizontal_run};
use crate::text::shaping::bidi::BidiParagraph;
use crate::text::shaping::cosmic::{cluster_flags, resolved_line_height};
use crate::text::shaping::fallback_spans::FallbackTextSpan;
use crate::text::shaping::itemize::{
    LogicalSegment, logical_segments_for_line, restore_backend_cluster_logical_order,
    virtual_hard_break_glyph,
};
use crate::text::shaping::line_break::LineBreakOpportunityMap;
use crate::text::shaping::script_segment::{script_segments, shaped_script_for_cluster};

pub(in crate::text::shaping) fn shape_horizontal_request(
    request: BackendShapeRequest<'_>,
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[FallbackTextSpan],
    database: &FontDatabase,
) -> Option<ShapedGlyphRun> {
    let line_breaks = LineBreakOpportunityMap::new(request.text);
    let scripts = script_segments(request.text);
    let line_height = resolved_line_height(request);
    let mut lines = Vec::new();

    for (line_index, hard_line) in crate::text::hard_lines(request.text)
        .into_iter()
        .enumerate()
    {
        let line_range = hard_line.content.clone();
        let mut glyphs = Vec::new();
        let mut line_metrics = HorizontalLineMetrics::default();
        let segments = logical_segments_for_line(
            request.text,
            line_range.clone(),
            fallback_spans,
            &scripts,
            bidi,
            None,
        )?;
        for segment in segments {
            let face = segment.face;
            glyphs.extend(shape_segment(
                request,
                line_range.start,
                segment,
                &line_breaks,
                database,
            )?);
            line_metrics.include_face(database, face, request.style.font_size);
        }
        if let Some(separator) = virtual_hard_break_glyph(request, &hard_line, bidi, &scripts) {
            glyphs.push(separator);
        }

        let mut cursor = 0.0_f32;
        for glyph in &mut glyphs {
            glyph.x = cursor;
            cursor += glyph.advance.max(0.0);
        }
        let (baseline, actual_line_height) =
            line_metrics.resolve(line_height, request.style.font_size.max(1.0));
        let full_range = hard_line.source_range();
        lines.push(ShapedTextLine {
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
            baseline,
            line_height: actual_line_height,
            glyphs,
        });
    }

    let measured_width = lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    Some(ShapedGlyphRun {
        source_text: request.shared_source_text(),
        source_range: request.source_range,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width,
        measured_height: lines.iter().map(|line| line.line_height).sum::<f32>(),
        lines,
    })
}

#[derive(Default)]
struct HorizontalLineMetrics {
    ascent: f32,
    descent: f32,
    line_gap: f32,
    has_face_metrics: bool,
}

impl HorizontalLineMetrics {
    fn include_face(&mut self, database: &FontDatabase, face: FontFaceId, font_size: f32) {
        let Some(metrics) = database.face_metrics(face).ok().flatten() else {
            return;
        };
        if metrics.units_per_em == 0 {
            return;
        }
        let scale = font_size.max(1.0) / f32::from(metrics.units_per_em);
        let use_windows_metrics = !metrics.uses_typographic_metrics
            && metrics.windows_ascender > 0
            && metrics.windows_descender > 0;
        let (ascent, descent) = if use_windows_metrics {
            (
                f32::from(metrics.windows_ascender) * scale,
                f32::from(metrics.windows_descender) * scale,
            )
        } else {
            (
                f32::from(metrics.ascender.max(0)) * scale,
                f32::from(metrics.descender.saturating_neg().max(0)) * scale,
            )
        };
        self.ascent = self.ascent.max(ascent);
        self.descent = self.descent.max(descent);
        self.line_gap = self
            .line_gap
            .max(f32::from(metrics.line_gap.max(0)) * scale);
        self.has_face_metrics = true;
    }

    fn resolve(&self, requested_line_height: f32, font_size: f32) -> (f32, f32) {
        if !self.has_face_metrics {
            return (font_size.max(1.0) * 0.8, requested_line_height);
        }
        let content_height = self.ascent + self.descent;
        let line_height = requested_line_height.max(content_height + self.line_gap);
        let leading = (line_height - content_height).max(0.0) * 0.5;
        (leading + self.ascent, line_height)
    }
}

fn shape_segment(
    request: BackendShapeRequest<'_>,
    line_start: usize,
    segment: LogicalSegment,
    line_breaks: &LineBreakOpportunityMap,
    database: &FontDatabase,
) -> Option<Vec<ShapedGlyph>> {
    let text = request.text.get(segment.range.start..segment.range.end)?;
    let mut backend = shape_horizontal_run(
        database,
        segment.face,
        segment.instance,
        text,
        segment.direction,
        segment.script.iso15924.as_str(),
        request.language,
        request.features(),
        request.include_kerning,
        crate::text::TextStyle::normalized_font_weight(request.style.font_weight),
        request.style.font_size,
    )?;
    valid_backend_run(&backend, text)?;
    restore_backend_cluster_logical_order(&mut backend.glyphs, segment.direction, |glyph| {
        glyph.source_offset
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
        let cluster_text = request.text.get(local_range.start..local_range.end)?;
        for (cluster_glyph_index, backend_glyph) in backend.glyphs[backend_start..backend_end]
            .iter()
            .copied()
            .enumerate()
        {
            let cluster_start = cluster_glyph_index == 0;
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
                cluster_flags: cluster_flags(
                    cluster_text,
                    segment.direction,
                    cluster_start,
                    if cluster_start {
                        line_breaks.flags_for_cluster(local_range.start, local_range.end)
                    } else {
                        Default::default()
                    },
                ),
                rotation: crate::text::ShapedGlyphRotation::None,
                script: shaped_script_for_cluster(cluster_text, segment.script),
            });
        }
        backend_start = backend_end;
    }
    Some(glyphs)
}

fn valid_backend_run(run: &HorizontalBackendRun, text: &str) -> Option<()> {
    (!run.glyphs.is_empty()
        && run.glyphs.iter().all(|glyph| {
            glyph.source_offset < text.len()
                && text.is_char_boundary(glyph.source_offset)
                && glyph.advance.is_finite()
                && glyph.x_offset.is_finite()
                && glyph.y_offset.is_finite()
        }))
    .then_some(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::HorizontalLineMetrics;
    use crate::text::font::FontDatabase;

    #[test]
    fn direct_line_metrics_use_scaled_actual_face_ascent() {
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
        let mut metrics = HorizontalLineMetrics::default();
        metrics.include_face(&database, face, 20.0);
        let (baseline, line_height) = metrics.resolve(24.0, 20.0);
        let expected_ascent = f32::from(source_metrics.ascender.max(0)) * 20.0
            / f32::from(source_metrics.units_per_em);

        assert!(baseline >= expected_ascent);
        assert!(line_height >= 24.0);
        assert!((baseline - 16.0).abs() > 0.01);
    }
}
