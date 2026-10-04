//! 页签在局部偏移后安排背景、选中下划线与文字；未相交时返回给上层已接管但无命令。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_segmented_control_geometry::{
    tab_font_size, tab_label_rect, tab_line_height, tab_paint_rect, tab_underline_rect,
};
use super::style::{tab_style, tab_text_color};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

/// 已被组件家族确认为页签后调用；相交才能出命令，选中时强调线位于底部而标签仍由共享文字来源决定。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_workbench_tab(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = tab_paint_rect(node, rect);
    if intersect(&rect, clip).is_none() {
        return;
    }
    let style = tab_style(node);
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(background),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    if node.checked || node.selected {
        let underline = tab_underline_rect(&rect);
        if intersect(&underline, clip).is_some() {
            commands.push(HostPaintCommand::quad(
                underline,
                Some(clip.clone()),
                order + 2,
                Some(style.selected_underline),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        let label_rect = tab_label_rect(&rect);
        if intersect(&label_rect, clip).is_some() {
            commands.push(HostPaintCommand::text(
                label_rect,
                Some(clip.clone()),
                order + 3,
                label,
                tab_text_color(node),
                tab_font_size(),
                tab_line_height(),
                UiTextRunPaintStyle::default(),
                opacity,
            ));
        }
    }
}
