use crate::text::sdf::SdfBakeParams;

use super::{SdfGlyphMetrics, FALLBACK_ADVANCE_RATIO};

pub(super) fn glyph_metrics(
    font: &ttf_parser::Face<'_>,
    px: f32,
    glyph: ttf_parser::GlyphId,
) -> SdfGlyphMetrics {
    let scale = px * (1.0 / font.units_per_em().max(1) as f32);
    // Preserve the pre-generation SDF measurement padding used by persisted
    // text layouts. Actual fdsm glyphs keep their bake-parameter-based metrics.
    let radius = (255.0 * scale).ceil() as u32 + 1;
    let bounds = font.glyph_bounding_box(glyph);
    let (left, bottom, width, height) = bounds.map_or((0.0, 0.0, 0, 0), |bounds| {
        (
            (bounds.x_min as f32 * scale).floor(),
            (bounds.y_min as f32 * scale).floor(),
            padded_extent(bounds.x_max.abs_diff(bounds.x_min), scale, radius),
            padded_extent(bounds.y_max.abs_diff(bounds.y_min), scale, radius),
        )
    });
    SdfGlyphMetrics {
        bitmap_width: width,
        bitmap_height: height,
        bitmap_left: left - radius as f32,
        bitmap_bottom: bottom - radius as f32,
        advance: (font.glyph_hor_advance(glyph).unwrap_or_default() as f32 * scale)
            .max(px * FALLBACK_ADVANCE_RATIO),
        ascent: font.ascender() as f32 * scale,
    }
}

fn padded_extent(extent: u16, scale: f32, radius: u32) -> u32 {
    if extent == 0 {
        0
    } else {
        (extent as f32 * scale) as u32 + 2 * radius
    }
}

pub(crate) fn scale_sdf_metrics_for_display(
    metrics: SdfGlyphMetrics,
    display_px: f32,
    bake_params: SdfBakeParams,
) -> SdfGlyphMetrics {
    let scale = display_px.max(1.0) / bake_params.bake_em_px_f32();
    SdfGlyphMetrics {
        bitmap_width: scale_bitmap_dimension(metrics.bitmap_width, scale),
        bitmap_height: scale_bitmap_dimension(metrics.bitmap_height, scale),
        bitmap_left: metrics.bitmap_left * scale,
        bitmap_bottom: metrics.bitmap_bottom * scale,
        advance: metrics.advance * scale,
        ascent: metrics.ascent * scale,
    }
}

fn scale_bitmap_dimension(value: u32, scale: f32) -> u32 {
    if value == 0 {
        0
    } else {
        ((value as f32 * scale).round() as u32).max(1)
    }
}

pub(super) fn fallback_metrics(px: f32) -> SdfGlyphMetrics {
    SdfGlyphMetrics {
        advance: px.max(1.0) * FALLBACK_ADVANCE_RATIO,
        ascent: px.max(1.0),
        ..SdfGlyphMetrics::default()
    }
}

#[cfg(test)]
#[path = "tests/glyph_metrics.rs"]
mod tests;
