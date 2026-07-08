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
mod tests {
    use super::{clamped_text_metrics, runtime_text_layout_frame};
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn retained_text_metrics_preserve_runtime_values_within_frame() {
        assert_eq!(clamped_text_metrics(18.0, 12.0, 14.0), (12.0, 14.0));
    }

    #[test]
    fn retained_text_metrics_clamp_to_minimum_and_frame_height() {
        assert_eq!(clamped_text_metrics(0.0, 0.0, 0.0), (1.0, 1.0));
        assert_eq!(clamped_text_metrics(10.0, 12.0, 14.0), (10.0, 10.0));
    }

    #[test]
    fn runtime_layout_frame_uses_minimum_positive_extent() {
        let frame = runtime_text_layout_frame(
            &FrameRect {
                x: 20.0,
                y: 30.0,
                width: 0.0,
                height: 8.0,
            },
            0.0,
        );

        assert_eq!(frame.x, 0.0);
        assert_eq!(frame.y, 0.0);
        assert_eq!(frame.width, 1.0);
        assert_eq!(frame.height, 1.0);
    }
}
