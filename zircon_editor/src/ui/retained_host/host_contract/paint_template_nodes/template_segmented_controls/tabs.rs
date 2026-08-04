use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_segmented_control_geometry::{
    tab_font_size, tab_label_rect, tab_line_height, tab_paint_rect, tab_underline_rect,
};
use super::style::{tab_style, tab_text_color};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
