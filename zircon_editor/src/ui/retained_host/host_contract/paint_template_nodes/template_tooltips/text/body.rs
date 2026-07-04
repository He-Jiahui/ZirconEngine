use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::tooltip_metrics;
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

    commands.push(HostPaintCommand::text(
        FrameRect {
            x: bubble.x + metrics.text_left,
            y: bubble.y + metrics.body_top,
            width: text_width,
            height: metrics.body_line_height,
        },
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

fn tooltip_body(node: &TemplatePaneNodeData) -> String {
    let text = node.label_text.as_str().trim();
    if text.is_empty() {
        "This is a tooltip".to_string()
    } else {
        text.to_string()
    }
}
