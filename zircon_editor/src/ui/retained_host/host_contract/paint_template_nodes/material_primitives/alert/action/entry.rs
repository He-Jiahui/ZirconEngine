use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::alert_action_frame;
use super::super::identity::alert_has_close_action;
use super::super::style::alert_action_color;
use super::close::push_alert_close_mark;
use super::line::push_alert_action_line;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = alert_action_frame(rect);
    let color = alert_action_color(node);
    if alert_has_close_action(node) {
        push_alert_close_mark(commands, &frame, clip, order, color, opacity);
    } else {
        push_alert_action_line(commands, &frame, clip, order, color, opacity);
    }
}
