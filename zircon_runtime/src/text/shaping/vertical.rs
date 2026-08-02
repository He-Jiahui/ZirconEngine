use std::collections::HashMap;

use crate::text::font::FontDatabase;
use crate::text::{BackendShapeRequest, FontFaceId, ShapedGlyph, ShapedGlyphRun, TextOrientation};

#[path = "vertical/orientation.rs"]
mod orientation;

#[path = "vertical/backend.rs"]
mod backend;

#[path = "vertical/direct.rs"]
mod direct;

#[cfg(test)]
#[path = "vertical/tests.rs"]
mod tests;

use orientation::vertical_glyph_metrics;

pub(in crate::text::shaping) use orientation::{
    vertical_shape_orientation, VerticalShapeOrientation,
};

pub(super) use direct::shape_vertical_request;

pub(crate) fn vertical_glyph_rotation(
    mode: crate::text::VerticalMode,
    cluster_text: &str,
) -> crate::text::ShapedGlyphRotation {
    orientation::vertical_glyph_rotation(mode, cluster_text)
}

pub(crate) fn vertical_glyph_advance(
    mode: crate::text::VerticalMode,
    cluster_text: &str,
    horizontal_advance: f32,
    font_size: f32,
) -> f32 {
    orientation::vertical_glyph_metrics(mode, cluster_text, horizontal_advance, font_size, None)
        .advance
}

pub(super) fn apply_vertical_layout(
    shaped: &mut ShapedGlyphRun,
    request: BackendShapeRequest<'_>,
    font_database: Option<&FontDatabase>,
) {
    let mut vertical_metrics = HashMap::new();
    apply_vertical_layout_with_native_metrics(shaped, request, |_, _, glyph| {
        let face = glyph.font_id?;
        let metrics = cached_vertical_metrics(
            &mut vertical_metrics,
            font_database?,
            face,
            request.style.font_size,
        )?;
        metrics.glyph_advance_px(glyph.glyph_id)
    });
}

fn cached_vertical_metrics<'a>(
    cache: &mut HashMap<FontFaceId, crate::text::font::FontVerticalMetrics<'a>>,
    database: &'a FontDatabase,
    face: FontFaceId,
    font_size: f32,
) -> Option<crate::text::font::FontVerticalMetrics<'a>> {
    if let Some(metrics) = cache.get(&face) {
        return Some(*metrics);
    }
    let metrics = database.vertical_metrics(face, font_size)?;
    cache.insert(face, metrics);
    Some(metrics)
}

fn apply_vertical_layout_with_native_metrics(
    shaped: &mut ShapedGlyphRun,
    request: BackendShapeRequest<'_>,
    mut vertical_advance_for_glyph: impl FnMut(usize, usize, &ShapedGlyph) -> Option<f32>,
) {
    if !matches!(request.orientation, TextOrientation::Vertical) {
        return;
    }

    let column_width = request.style.font_size.max(1.0);
    let mut max_column_height = 0.0_f32;
    let mut populated_columns = 0_usize;
    for (line_index, line) in shaped.lines.iter_mut().enumerate() {
        let mut cursor_y = 0.0_f32;
        let mut glyph_index = 0_usize;
        while glyph_index < line.glyphs.len() {
            let cluster_range = line.glyphs[glyph_index].source_range;
            let mut cluster_end = glyph_index + 1;
            while cluster_end < line.glyphs.len()
                && line.glyphs[cluster_end].source_range == cluster_range
            {
                cluster_end += 1;
            }

            let cluster_text = source_cluster_text(request, cluster_range);
            let horizontal_advance = line.glyphs[glyph_index..cluster_end]
                .iter()
                .map(|glyph| glyph.advance.max(0.0))
                .sum::<f32>();
            let native_vertical_advance = line.glyphs[glyph_index..cluster_end]
                .iter()
                .enumerate()
                .find_map(|(cluster_glyph_index, glyph)| {
                    vertical_advance_for_glyph(line_index, glyph_index + cluster_glyph_index, glyph)
                });
            let metrics = vertical_glyph_metrics(
                request.vertical_mode,
                cluster_text,
                horizontal_advance,
                column_width,
                native_vertical_advance,
            );
            for (cluster_glyph_index, glyph) in
                line.glyphs[glyph_index..cluster_end].iter_mut().enumerate()
            {
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
            glyph_index = cluster_end;
        }

        if !line.glyphs.is_empty() {
            populated_columns += 1;
        }
        line.measured_width = cursor_y;
        line.baseline = column_width * 0.5;
        line.line_height = column_width;
        max_column_height = max_column_height.max(cursor_y);
    }

    shaped.measured_width = populated_columns as f32 * column_width;
    shaped.measured_height = max_column_height;
}

pub(super) fn source_cluster_text<'a>(
    request: BackendShapeRequest<'a>,
    source_range: crate::text::TextRange,
) -> &'a str {
    let start = source_range
        .start
        .saturating_sub(request.source_range.start)
        .min(request.text.len());
    let end = source_range
        .end
        .saturating_sub(request.source_range.start)
        .clamp(start, request.text.len());
    request.text.get(start..end).unwrap_or_default()
}
