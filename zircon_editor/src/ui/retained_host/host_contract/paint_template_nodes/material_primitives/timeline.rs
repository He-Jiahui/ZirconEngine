mod connector;
mod dot;
mod geometry;
mod identity;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use connector::push_timeline_connector;
use dot::push_timeline_dot;
use identity::{timeline_primitive_kind, TimelinePrimitiveKind};

/// 时间线节点先按角色接管并做裁剪；separator 不发绘制命令，但仍阻止通用表面重复绘制。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_timeline_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = timeline_primitive_kind(node) else {
        return false;
    };
    if intersect(rect, clip).is_none() {
        return true;
    }
    match kind {
        TimelinePrimitiveKind::Dot => {
            push_timeline_dot(commands, node, rect, clip, order, opacity);
        }
        TimelinePrimitiveKind::Connector => {
            push_timeline_connector(commands, node, rect, clip, order, opacity);
        }
        TimelinePrimitiveKind::Separator => {}
    }
    true
}

#[cfg(test)]
#[path = "tests/timeline_optimization_batch_ha_editor582_tests.rs"]
mod optimization_batch_ha_editor582_tests;
