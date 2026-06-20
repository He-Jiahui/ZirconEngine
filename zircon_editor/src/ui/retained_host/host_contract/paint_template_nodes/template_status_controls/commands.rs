use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::chips::push_status_chip;
use super::icons::push_status_icon_button;
use super::identity::{status_control_kind, StatusControlKind};
use super::signals::push_status_signal_item;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match status_control_kind(node) {
        Some(StatusControlKind::Signal(kind)) => {
            push_status_signal_item(commands, node, rect, clip, order, kind, opacity);
            true
        }
        Some(StatusControlKind::Chip) => {
            push_status_chip(commands, node, rect, clip, order, opacity);
            true
        }
        Some(StatusControlKind::Icon(kind)) => {
            push_status_icon_button(commands, node, rect, clip, order, kind, opacity);
            true
        }
        None => false,
    }
}
