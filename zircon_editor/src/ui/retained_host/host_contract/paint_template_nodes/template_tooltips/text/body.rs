use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::frame_is_within;
use super::super::metrics::tooltip_metrics;
use super::super::text::tooltip_body;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_tooltip_body(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_width: f32,
    body_color: [u8; 4],
    opacity: f32,
) {
    let body = tooltip_body(node);
    if body.is_empty() {
        return;
    }
    let metrics = tooltip_metrics();

    let frame = FrameRect {
        x: bubble.x + metrics.text_left,
        y: bubble.y + metrics.body_top,
        width: text_width,
        height: metrics.body_line_height,
    };
    if !frame_is_within(bubble, &frame) {
        return;
    }
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        body,
        body_color,
        metrics.body_font_size,
        metrics.body_line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
