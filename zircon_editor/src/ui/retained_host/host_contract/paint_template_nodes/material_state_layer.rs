//! 在控件表面与内容之间投影交互反馈；完整状态层与按压涟漪是两个独立开关。
//! 调用方先确认表面与父裁剪相交，并预留反馈层级；这里不接管命中测试或动画时间。

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

mod ripple;
mod state;

use ripple::{push_ripple_commands, ripple_is_visible};
use state::{state_layer_color, state_layer_opacity};

#[cfg(test)]
use ripple::{ripple_diameter, ripple_rect, RIPPLE_DIAMETER_EXPANSION};

/// 按按钮/通用表面的已解析状态追加反馈，传入的是宿主坐标和当前节点的父裁剪。
/// 反馈不改变内容布局；外部透明度同时作用于状态层和涟漪，禁用状态仍可产生完整状态层。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_state_layer_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    corner_radius: f32,
    order: i32,
    opacity_multiplier: f32,
) {
    let overlay_opacity = state_layer_opacity(node);
    let ripple_visible = ripple_is_visible(node);
    if overlay_opacity.is_none() && !ripple_visible {
        return;
    }
    if !ripple_visible && intersect(rect, clip).is_none() {
        return;
    }

    let color = state_layer_color(node);
    if let Some(opacity) = overlay_opacity {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            corner_radius,
            opacity * opacity_multiplier,
        ));
    }

    if ripple_visible {
        push_ripple_commands(
            commands,
            node,
            rect,
            clip,
            order + 1,
            color,
            opacity_multiplier,
        );
    }
}

#[cfg(test)]
#[path = "material_state_layer_tests/tests/mod.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/material_state_layer_optimization_batch_hb_editor583_tests.rs"]
mod optimization_batch_hb_editor583_tests;
