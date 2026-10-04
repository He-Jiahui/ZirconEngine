//! 树行选中/交互底面与层级导线的绘制分工；导线可在无底色的普通行中仍保持树关系可读。
//! 命令使用调用方裁剪，底面是否存在由状态样式决定。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{
    tree_guide_color, tree_guide_opacity, tree_guide_rect, tree_row_radius,
};
use super::style::{tree_row_background, tree_row_border, tree_row_border_width};

const TREE_ROW_SURFACE_COMMAND_CAPACITY: usize = 1;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = tree_row_background(node) else {
        return;
    };
    commands.reserve(TREE_ROW_SURFACE_COMMAND_CAPACITY);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        tree_row_border(node),
        tree_row_border_width(node),
        tree_row_radius(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_indent_guides(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let depth = node.tree_depth.max(0) as usize;
    // TODO: [CR-EDITOR-PAINT-ROWS-0001] tree_depth 来自可编写整数属性，当前按整个深度预留并生成导线。
    // 需确认投影是否保证深度上限；深导线即使在行外也先占命令容量，裁剪不会限制 CPU 工作量。
    commands.reserve(depth);
    let guide_color = tree_guide_color();
    let guide_opacity = tree_guide_opacity();
    for level in 0..depth {
        commands.push(HostPaintCommand::quad(
            tree_guide_rect(rect, level),
            Some(clip.clone()),
            order,
            Some(guide_color),
            None,
            0.0,
            0.0,
            opacity * guide_opacity,
        ));
    }
}

#[cfg(test)]
#[path = "tests/surface_optimization_tests.rs"]
mod optimization_tests;
