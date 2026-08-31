use std::sync::atomic::Ordering;

use crate::core::framework::text::{
    TextDirection, TextGlyph, TextGlyphFlags, TextGlyphRotation, TextLayoutMetrics,
    TextShapeResult, TextShapeRun,
};
use crate::text::font::{FontCollectionSnapshot, register_font_handle_batch_for_collection};
use crate::text::{ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun};

#[cfg(test)]
use super::CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT;
use super::generation_retry_metrics;

pub(super) fn project_shape_result(
    shaped: ShapedGlyphRun,
    resolved_direction: TextDirection,
    font_collection: &FontCollectionSnapshot,
) -> TextShapeResult {
    record_neutral_projection(&shaped);
    let metrics = TextLayoutMetrics {
        width: shaped.measured_width,
        height: shaped.measured_height,
        ascent: shaped.lines.first().map_or(0.0, |line| line.baseline),
        descent: shaped
            .lines
            .first()
            .map_or(0.0, |line| (line.line_height - line.baseline).max(0.0)),
        line_gap: 0.0,
        baseline: shaped.lines.first().map_or(0.0, |line| line.baseline),
    };
    let font_handles = register_font_handle_batch_for_collection(
        font_collection.service(),
        &shaped
            .lines
            .iter()
            .flat_map(|line| {
                line.glyphs
                    .iter()
                    .map(|glyph| (glyph.font_id, glyph.font_instance_id))
            })
            .collect::<Vec<_>>(),
        font_collection.generation(),
    );
    let mut font_handles = font_handles.into_iter();
    let runs = shaped
        .lines
        .into_iter()
        .map(|line| TextShapeRun {
            source_range: line.source_range.start..line.source_range.end,
            direction: line
                .glyphs
                .first()
                .map_or(resolved_direction, |glyph| glyph.direction),
            glyphs: line
                .glyphs
                .into_iter()
                .map(|glyph| {
                    // A malformed projection batch must not terminate text rendering. Missing
                    // handles take the existing fail-closed raster path for this glyph.
                    project_glyph(&glyph, font_handles.next().unwrap_or_default())
                })
                .collect(),
        })
        .collect();
    TextShapeResult {
        runs,
        metrics,
        resolved_direction,
    }
}

fn record_neutral_projection(shaped: &ShapedGlyphRun) {
    #[cfg(test)]
    CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT.set(
        CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT
            .get()
            .saturating_add(1),
    );
    let glyph_count = shaped
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    let projected_bytes = shaped
        .lines
        .len()
        .saturating_mul(std::mem::size_of::<TextShapeRun>())
        .saturating_add(glyph_count.saturating_mul(std::mem::size_of::<TextGlyph>()));
    let metrics = generation_retry_metrics();
    metrics
        .neutral_projection_count
        .fetch_add(1, Ordering::Relaxed);
    metrics
        .neutral_projection_glyph_count
        .fetch_add(glyph_count as u64, Ordering::Relaxed);
    metrics
        .neutral_projection_bytes
        .fetch_add(projected_bytes as u64, Ordering::Relaxed);
}

pub(crate) fn project_glyph(
    glyph: &ShapedGlyph,
    (font_face, font_instance): (
        Option<crate::core::framework::text::TextFontFaceHandle>,
        Option<crate::core::framework::text::TextFontFaceHandle>,
    ),
) -> TextGlyph {
    TextGlyph {
        glyph_id: glyph.glyph_id,
        source_range: glyph.source_range.start..glyph.source_range.end,
        visual_range: glyph.visual_range.start..glyph.visual_range.end,
        advance: glyph.advance,
        position: [glyph.x, glyph.y],
        offset: [glyph.offset_x, glyph.offset_y],
        font_face,
        font_instance,
        rotation: match glyph.rotation {
            ShapedGlyphRotation::None => TextGlyphRotation::None,
            ShapedGlyphRotation::Cw90 => TextGlyphRotation::Clockwise90,
        },
        bidi_level: glyph.bidi_level,
        flags: TextGlyphFlags {
            cluster_start: glyph.cluster_flags.cluster_start,
            right_to_left: glyph.cluster_flags.rtl,
            whitespace: glyph.cluster_flags.whitespace,
            space: glyph.cluster_flags.space,
            tab: glyph.cluster_flags.tab,
            mandatory_break: glyph.cluster_flags.mandatory_break,
            soft_break: glyph.cluster_flags.soft_break,
            virtual_glyph: glyph.cluster_flags.virtual_glyph,
            vertical_decision: glyph.cluster_flags.vertical_decision,
        },
        requires_rasterization: font_face.is_some()
            && !glyph.cluster_flags.virtual_glyph
            && !glyph.cluster_flags.whitespace
            && !glyph.cluster_flags.space
            && !glyph.cluster_flags.tab,
    }
}
