use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::centered_square;
use super::style::{
    timeline_dot_background_color, timeline_dot_border_color, timeline_dot_border_width,
    timeline_dot_is_outlined, timeline_dot_tone_color,
};

pub(super) fn push_timeline_dot(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let dot = centered_square(rect);
    if dot.width <= 0.0 || dot.height <= 0.0 {
        return;
    }

    let outlined = timeline_dot_is_outlined(node);
    let tone = timeline_dot_tone_color(node);
    let background = timeline_dot_background_color(node, outlined, tone);
    let border_color = timeline_dot_border_color(node, outlined, tone);
    let border_width = timeline_dot_border_width(node, outlined, border_color.is_some());
    commands.push(HostPaintCommand::quad(
        dot.clone(),
        Some(clip.clone()),
        order,
        background,
        border_color,
        border_width,
        dot.width.min(dot.height) * 0.5,
        opacity,
    ));
}
