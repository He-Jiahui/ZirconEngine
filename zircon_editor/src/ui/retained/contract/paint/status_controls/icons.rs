//! 图标按钮沿用状态选择器处理透明/选中/焦点色；图像资源最终通过共享 icon 资产入口。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_icon_button_style;
use super::super::template_status_control_geometry::{
    frame_is_within, has_paintable_status_control_extent, status_control_offset_rect,
    status_icon_button_glyph_rect, status_icon_button_radius, workbench_status_metrics,
};
use super::super::template_status_glyphs::{push_status_icon_glyph, StatusIconKind};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_icon_button(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    opacity: f32,
) {
    let parent_rect = rect;
    let control_rect = status_control_offset_rect(node, parent_rect);
    // BUG: [CR-EDITOR-PAINT-CONTROLSTYLE-0004] 右侧图标继承非零视觉偏移后，等尺寸的 control_rect 不可能包含于原 parent_rect；
    // 这里提前退出，SnapToggle/World/Target 的图标和表面均消失；正式状态栏声明偏移 -0.5。
    if !has_paintable_status_control_extent(&control_rect)
        || intersect(&control_rect, clip).is_none()
    {
        return;
    }
    let style = select_workbench_status_icon_button_style(node);
    push_status_icon_surface(
        commands,
        &control_rect,
        clip,
        order,
        style.background,
        style.border,
        opacity,
    );
    let glyph = status_icon_button_glyph_rect(&control_rect);
    if !frame_is_within(&control_rect, &glyph) || intersect(&glyph, clip).is_none() {
        return;
    }
    push_status_icon_glyph(
        commands,
        &glyph,
        clip,
        order + 2,
        kind,
        style.glyph,
        opacity,
    );
}

fn push_status_icon_surface(
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
        status_icon_button_radius(),
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
    fn shifted_status_icon_paints_inside_host_clip() {
        let mut node = TemplatePaneNodeData::default();
        node.checked = true;
        node.selected = true;
        node.layout_offset_y = -0.5;
        let parent = FrameRect {
            x: 10.0,
            y: 10.0,
            width: 36.0,
            height: 36.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
        };
        let mut commands = Vec::new();
        push_status_icon_button(
            &mut commands,
            &node,
            &parent,
            &clip,
            0,
            StatusIconKind::World,
            1.0,
        );
        assert!(
            !commands.is_empty(),
            "a shifted status action remains visible"
        );
        assert!(commands.iter().any(|c| c.frame.y < parent.y));
    }
}
