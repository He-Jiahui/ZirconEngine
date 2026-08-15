mod connector;
mod dot;
mod geometry;
mod identity;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use connector::push_timeline_connector;
use dot::push_timeline_dot;
use identity::{timeline_primitive_kind, TimelinePrimitiveKind};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_timeline_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match timeline_primitive_kind(node) {
        Some(TimelinePrimitiveKind::Dot) => {
            push_timeline_dot(commands, node, rect, clip, order, opacity);
        }
        Some(TimelinePrimitiveKind::Connector) => {
            push_timeline_connector(commands, node, rect, clip, order, opacity);
        }
        Some(TimelinePrimitiveKind::Separator) => {}
        None => return false,
    }
    true
}
