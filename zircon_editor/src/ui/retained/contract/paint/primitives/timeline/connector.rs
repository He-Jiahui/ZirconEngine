use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::style::timeline_connector_color;

const TIMELINE_CONNECTOR_WIDTH: f32 = 2.0;

pub(super) fn push_timeline_connector(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let width = rect.width.min(TIMELINE_CONNECTOR_WIDTH).max(0.0);
    if width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let x = (rect.x + (rect.width - width).max(0.0) * 0.5).round();
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y: rect.y,
            width,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(timeline_connector_color(node)),
        None,
        0.0,
        width * 0.5,
        opacity,
    ));
}
