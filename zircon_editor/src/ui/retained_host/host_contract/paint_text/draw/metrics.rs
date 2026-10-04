use zircon_runtime_interface::ui::layout::UiFrame;

use super::super::super::data::FrameRect;

const MIN_TEXT_METRIC_PX: f32 = 1.0;
const RUNTIME_LAYOUT_FRAME_ORIGIN_PX: f32 = 0.0;

pub(super) fn clamped_text_metrics(
    frame_height: f32,
    font_size: f32,
    line_height: f32,
) -> (f32, f32) {
    let max_text_height = frame_height.max(MIN_TEXT_METRIC_PX);
    let font_size = font_size.max(MIN_TEXT_METRIC_PX).min(max_text_height);
    let line_height = line_height
        .max(font_size)
        .max(MIN_TEXT_METRIC_PX)
        .min(max_text_height);
    (font_size, line_height)
}

pub(super) fn runtime_text_layout_frame(rect: &FrameRect, line_height: f32) -> UiFrame {
    UiFrame::new(
        RUNTIME_LAYOUT_FRAME_ORIGIN_PX,
        RUNTIME_LAYOUT_FRAME_ORIGIN_PX,
        rect.width.max(MIN_TEXT_METRIC_PX),
        line_height.max(MIN_TEXT_METRIC_PX),
    )
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
