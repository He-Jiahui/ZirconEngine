use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::frame_is_within;
use super::super::metrics::tooltip_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_tooltip_title(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_width: f32,
    title_color: [u8; 4],
    opacity: f32,
) {
    let title = tooltip_title(node);
    if title.is_empty() {
        return;
    }
    let metrics = tooltip_metrics();

    let frame = FrameRect {
        x: bubble.x + metrics.text_left,
        y: bubble.y + metrics.title_top,
        width: text_width,
        height: metrics.title_line_height,
    };
    if !frame_is_within(bubble, &frame) {
        return;
    }
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        title,
        title_color,
        metrics.title_font_size,
        metrics.title_line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn tooltip_title(node: &TemplatePaneNodeData) -> String {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip".to_string()
    } else {
        text.to_string()
    }
}
