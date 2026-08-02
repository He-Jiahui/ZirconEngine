use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::is_visible_frame;
use super::super::paint_text::{draw_text_with_size_and_style, measure_runtime_text_width};
use super::super::paint_theme::{HostControlMetrics, current_host_metrics};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract) fn draw_text_bars(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    color: [u8; 4],
) {
    draw_text_bars_clipped(frame, x, y, text, None, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_text_bars_clipped(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    let metrics = current_host_metrics();
    draw_text_with_size_and_style(
        frame,
        text_bars_frame(x, y, text, metrics),
        text,
        clip,
        color,
        metrics.font_body,
        text_bar_line_height(metrics),
        UiTextRunPaintStyle::default(),
    );
}

fn text_bars_frame(x: f32, y: f32, text: &str, metrics: HostControlMetrics) -> FrameRect {
    FrameRect {
        x,
        y,
        width: text_bar_frame_width(
            measure_runtime_text_width(text, metrics.font_body),
            metrics.text_clip_guard,
        ),
        height: text_bar_line_height(metrics),
    }
}

fn text_bar_frame_width(measured_width: f32, clip_guard: f32) -> f32 {
    (measured_width + clip_guard.max(0.0)).max(1.0)
}

fn text_bar_line_height(metrics: HostControlMetrics) -> f32 {
    metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil())
}

#[cfg(test)]
mod tests {
    use super::{label_marker_frame, text_bar_frame_width, text_bar_line_height, text_bars_frame};
    use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

    #[test]
    fn text_bars_frame_uses_runtime_text_measurement() {
        let narrow = text_bars_frame(0.0, 0.0, "iiiiiiiiiiii", METRICS);
        let wide = text_bars_frame(0.0, 0.0, "WWWWWWWWWWWW", METRICS);

        assert!(
            wide.width > narrow.width + 8.0,
            "same-character-count text bars should follow runtime glyph width, narrow={narrow:?}, wide={wide:?}"
        );
        assert_eq!(wide.height, text_bar_line_height(METRICS));
    }

    #[test]
    fn text_bars_frame_reserves_trailing_glyph_clip_guard() {
        assert_eq!(text_bar_frame_width(80.0, 6.0), 86.0);
        assert_eq!(text_bar_frame_width(0.0, -4.0), 1.0);
    }

    #[test]
    fn label_marker_frame_uses_host_metrics_without_overflowing_the_target() {
        let target = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 60.0,
            height: 10.0,
        };
        let frame = label_marker_frame(&target, METRICS);

        assert_eq!(frame.x, target.x + METRICS.button_pad_x);
        assert!(frame.y >= target.y);
        assert!(frame.x + frame.width <= target.x + target.width);
        assert!(frame.y + frame.height <= target.y + target.height);

        let narrow = FrameRect {
            width: 12.0,
            ..target
        };
        let narrow_frame = label_marker_frame(&narrow, METRICS);
        assert!(narrow_frame.x + narrow_frame.width <= narrow.x + narrow.width);
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_label_marker(
    frame: &mut HostRgbaFrame,
    target: &FrameRect,
    label: &str,
    color: [u8; 4],
) {
    if !is_visible_frame(target) {
        return;
    }
    let metrics = current_host_metrics();
    draw_text_with_size_and_style(
        frame,
        label_marker_frame(target, metrics),
        label,
        Some(target),
        color,
        metrics.font_body,
        label_marker_line_height(target, metrics),
        UiTextRunPaintStyle::default(),
    );
}

fn label_marker_frame(target: &FrameRect, metrics: HostControlMetrics) -> FrameRect {
    let horizontal_inset = metrics
        .button_pad_x
        .max(0.0)
        .min(target.width.max(0.0) * 0.5);
    let line_height = label_marker_line_height(target, metrics);
    FrameRect {
        x: target.x + horizontal_inset,
        y: target.y + ((target.height - line_height).max(0.0) * 0.5),
        width: (target.width - horizontal_inset * 2.0).max(0.0),
        height: line_height,
    }
}

fn label_marker_line_height(target: &FrameRect, metrics: HostControlMetrics) -> f32 {
    metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil())
        .min(target.height.max(0.0))
}
