use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::tooltip_metrics;
use super::body::push_tooltip_body;
use super::title::push_tooltip_title;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    title_color: [u8; 4],
    body_color: [u8; 4],
    opacity: f32,
) {
    let metrics = tooltip_metrics();
    let text_width = (bubble.width - metrics.text_left * 2.0).max(1.0);
    push_tooltip_title(
        commands,
        node,
        bubble,
        clip,
        order,
        text_width,
        title_color,
        opacity,
    );
    push_tooltip_body(
        commands,
        node,
        bubble,
        clip,
        order + 1,
        text_width,
        body_color,
        opacity,
    );
}
