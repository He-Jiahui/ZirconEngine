use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::is_visible_frame;
use super::super::paint_text::{draw_text_with_size_and_style, measure_runtime_text_width};
use super::super::paint_theme::{current_host_metrics, HostControlMetrics};
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
#[path = "tests/text_markers.rs"]
mod tests;

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
