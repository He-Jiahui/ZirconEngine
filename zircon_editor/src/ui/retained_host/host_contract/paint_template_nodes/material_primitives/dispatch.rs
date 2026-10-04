use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{alert, avatar, badge, chip, divider, paper, skeleton, text_field, timeline};

/// 先于通用模板回退识别专用组件；返回 true 表示节点已由该族接管，即使裁剪后没有命令也应阻止重复表面。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if alert::push_alert_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if chip::push_chip_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if avatar::push_avatar_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if badge::push_badge_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if skeleton::push_skeleton_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if paper::push_paper_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if timeline::push_timeline_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if divider::push_divider_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    false
}

/// TextField 只替换表面层；外层仍须继续文本、图片和输入焦点的通用绘制链。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_text_field_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    text_field::push_text_field_surface_commands(commands, node, rect, clip, order, opacity)
}
