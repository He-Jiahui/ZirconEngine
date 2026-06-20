use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::action::push_alert_action;
use super::geometry::{alert_action_width, alert_message_left, alert_message_right, alert_rect};
use super::icon::push_alert_icon;
use super::identity::{alert_has_icon, is_alert_root_node, is_alert_slot_node};
use super::message::push_alert_message;
use super::surface::push_alert_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_alert_slot_node(node) {
        return true;
    }
    if !is_alert_root_node(node) {
        return false;
    }

    let paint_rect = alert_rect(rect);
    if paint_rect.width <= 0.0 || paint_rect.height <= 0.0 {
        return true;
    }

    push_alert_surface(commands, node, &paint_rect, clip, order, opacity);
    if alert_has_icon(node) {
        push_alert_icon(commands, node, &paint_rect, clip, order + 1, opacity);
    }
    let message_left = alert_message_left(node, &paint_rect);
    let message_right = alert_message_right(node, &paint_rect);
    let action_width = alert_action_width(node);
    push_alert_message(
        commands,
        node,
        &paint_rect,
        message_left,
        message_right,
        clip,
        order + 2,
        opacity,
    );
    if action_width > 0.0 {
        push_alert_action(commands, node, &paint_rect, clip, order + 3, opacity);
    }

    true
}
