//! chip 背景可完全透明，文字仍由状态样式选择器决定；视觉偏移来自模板属性并与信号项分开。

#[path = "chips/text.rs"]
mod text;

use self::text::push_status_chip_text_for_node;
use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_chip_style;
use super::super::template_node_labels::template_node_label;
use super::super::template_status_control_geometry::{
    has_paintable_status_control_extent, status_chip_radius, status_control_offset_rect,
    workbench_status_metrics,
};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_chip(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let parent_rect = rect;
    let control_rect = status_control_offset_rect(node, parent_rect);
    // BUG: [CR-EDITOR-PAINT-CONTROLSTYLE-0003] 状态栏右侧继承非零视觉偏移时，等尺寸的 control_rect 无法仍在 parent_rect 内；
    // 本分支会丢弃整个 chip。证据：unreal/components/host_status_bar.zui 的 status_right_offset_y=-0.5 传入 Grid/Snap/Zoom。
    if !has_paintable_status_control_extent(&control_rect)
        || intersect(&control_rect, clip).is_none()
    {
        return;
    }
    let style = select_workbench_status_chip_style(node);
    push_status_chip_surface(
        commands,
        &control_rect,
        clip,
        order,
        style.background,
        style.border,
        opacity,
    );

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        push_status_chip_text_for_node(
            commands,
            node,
            &control_rect,
            clip,
            order + 2,
            &label,
            style.label_text,
            style.value_text,
            opacity,
        );
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_text_colors(
    node: &TemplatePaneNodeData,
) -> ([u8; 4], [u8; 4]) {
    let style = select_workbench_status_chip_style(node);
    (style.label_text, style.value_text)
}

fn push_status_chip_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    background: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let background = visible_color(background);
    let border = visible_color(border);
    if background.is_none() && border.is_none() {
        return;
    }
    let border_width = if border.is_some() {
        workbench_status_metrics().border_width
    } else {
        0.0
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border,
        border_width,
        status_chip_radius(),
        opacity,
    ));
}

fn visible_color(color: [u8; 4]) -> Option<[u8; 4]> {
    (color[3] > 0).then_some(color)
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    #[test]
    fn shifted_status_chip_paints_inside_host_clip() {
        let mut node = TemplatePaneNodeData::default();
        node.text = "Grid: 4".into();
        node.checked = true;
        node.selected = true;
        node.layout_offset_y = -0.5;
        let parent = FrameRect {
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 36.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 80.0,
        };
        let mut commands = Vec::new();
        push_status_chip(&mut commands, &node, &parent, &clip, 0, 1.0);
        assert!(
            !commands.is_empty(),
            "a declared visual offset must not discard the chip"
        );
        assert!(commands.iter().any(|c| c.frame.y < parent.y));
    }
}
