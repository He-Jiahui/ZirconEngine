use crate::text::sdf::SdfBakeParams;

use super::{FALLBACK_ADVANCE_RATIO, SdfGlyphMetrics};

pub(super) fn glyph_metrics(
    font: &fontsdf::Font,
    px: f32,
    metrics: fontsdf::Metrics,
) -> SdfGlyphMetrics {
    let ascent = font
        .inner()
        .horizontal_line_metrics(px)
        .map(|metrics| metrics.ascent)
        .unwrap_or(px);
    SdfGlyphMetrics {
        bitmap_width: metrics.width as u32,
        bitmap_height: metrics.height as u32,
        bitmap_left: metrics.xmin as f32,
        bitmap_bottom: metrics.ymin as f32,
        advance: metrics.advance_width.max(px * FALLBACK_ADVANCE_RATIO),
        ascent,
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
