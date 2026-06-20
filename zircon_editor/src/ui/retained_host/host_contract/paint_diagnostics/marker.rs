use super::super::data::FrameRect;
use super::visibility::diagnostic_visible_frame;

const MARKER_HORIZONTAL_PADDING: f32 = 14.0;
const MARKER_RIGHT_INSET: f32 = 8.0;
const MARKER_TOP_INSET: f32 = 6.0;
const MARKER_VERTICAL_INSET: f32 = 12.0;
const MARKER_MIN_HEIGHT: f32 = 14.0;
const APPROX_GLYPH_WIDTH: f32 = 8.0;

pub(in crate::ui::retained_host::host_contract) fn debug_refresh_overlay_frame(
    top_bar: &FrameRect,
    label: &str,
) -> Option<FrameRect> {
    if label.trim().is_empty() || !diagnostic_visible_frame(top_bar) {
        return None;
    }
    let marker_width = (label.chars().count() as f32 * APPROX_GLYPH_WIDTH
        + MARKER_HORIZONTAL_PADDING)
        .min((top_bar.width - MARKER_VERTICAL_INSET).max(1.0))
        .max(1.0);
    Some(FrameRect {
        x: (top_bar.x + top_bar.width - marker_width - MARKER_RIGHT_INSET).max(top_bar.x),
        y: top_bar.y + MARKER_TOP_INSET,
        width: marker_width,
        height: (top_bar.height - MARKER_VERTICAL_INSET).max(MARKER_MIN_HEIGHT),
    })
}
