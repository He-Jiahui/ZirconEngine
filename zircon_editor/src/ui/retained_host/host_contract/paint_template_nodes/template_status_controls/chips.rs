mod text;

use self::text::push_status_chip_text;
use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_chip_style;
use super::super::template_node_labels::template_node_label;
use super::super::template_status_control_geometry::{
    status_chip_radius, status_control_offset_rect, workbench_status_metrics,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_chip(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    let style = select_workbench_status_chip_style(node);
    push_status_chip_surface(
        commands,
        &rect,
        clip,
        order,
        style.background,
        style.border,
        opacity,
    );

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        push_status_chip_text(
            commands,
            &rect,
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
