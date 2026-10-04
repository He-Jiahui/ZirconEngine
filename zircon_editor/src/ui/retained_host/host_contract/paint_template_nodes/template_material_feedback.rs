//! 专用反馈控件先于通用表面被识别，避免进度条或遮罩又生成一份通用背景。

mod backdrop;
mod circular_progress;
mod metrics;
mod palette;
mod progress;
mod state;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;
use backdrop::{is_material_backdrop_node, push_material_backdrop_commands};
use progress::push_material_progress_commands;
use state::is_material_progress_node;

/// 返回值表示已认领节点类型；完全裁掉、关闭或不可见时也返回 true，禁止后续 fallback 再绘制。
/// 上游已完成坐标变换与过渡透明度解析，此入口不参与交互状态更新。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_feedback_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let is_backdrop = is_material_backdrop_node(node);
    let is_progress = is_material_progress_node(node);
    if !is_backdrop && !is_progress {
        return false;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };
    if is_backdrop {
        push_material_backdrop_commands(commands, node, rect, &clip, order, opacity);
    } else {
        push_material_progress_commands(commands, node, rect, &clip, order, opacity);
    }
    true
}

#[cfg(test)]
#[path = "template_material_feedback_tests/tests/mod.rs"]
mod tests;
