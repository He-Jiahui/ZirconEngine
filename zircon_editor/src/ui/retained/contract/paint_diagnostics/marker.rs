use super::super::data::FrameRect;
use super::super::paint_text::measure_runtime_text_width;
use super::visibility::diagnostic_visible_frame;

const MARKER_HORIZONTAL_PADDING: f32 = 14.0;
const MARKER_RIGHT_INSET: f32 = 8.0;
const MARKER_TOP_INSET: f32 = 6.0;
const MARKER_VERTICAL_INSET: f32 = 12.0;
const MARKER_MIN_HEIGHT: f32 = 14.0;
const MARKER_FONT_SIZE: f32 = 12.0;

pub(in crate::ui::retained_host::host_contract) fn debug_refresh_overlay_frame(
    top_bar: &FrameRect,
    label: &str,
) -> Option<FrameRect> {
    if label.trim().is_empty() || !diagnostic_visible_frame(top_bar) {
        return None;
    }
    let marker_width = (measure_runtime_text_width(label, MARKER_FONT_SIZE)
        + MARKER_HORIZONTAL_PADDING)
        .min((top_bar.width - MARKER_VERTICAL_INSET).max(1.0))
        .max(1.0);
    let y = top_bar.y + MARKER_TOP_INSET;
    let available_height = top_bar.y + top_bar.height - y;
    if available_height <= 0.0 {
        return None;
    }
    let marker_height = (top_bar.height - MARKER_VERTICAL_INSET)
        .max(MARKER_MIN_HEIGHT)
        .min(available_height);
    Some(FrameRect {
        x: (top_bar.x + top_bar.width - marker_width - MARKER_RIGHT_INSET).max(top_bar.x),
        y,
        width: marker_width,
        height: marker_height,
    })
}
