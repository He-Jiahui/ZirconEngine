use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_status_chip_style;
use super::super::template_node_labels::template_node_label;
use super::super::template_status_control_geometry::{
    status_chip_chevron_rect, status_chip_text_rect, status_control_offset_rect,
    status_line_height, STATUS_CHIP_RADIUS, STATUS_FONT_SIZE,
};
use super::super::template_status_glyphs::push_down_chevron;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.background),
        Some(style.border),
        1.0,
        STATUS_CHIP_RADIUS,
        opacity,
    ));

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        commands.push(HostPaintCommand::text(
            status_chip_text_rect(&rect),
            Some(clip.clone()),
            order + 2,
            label,
            style.text,
            STATUS_FONT_SIZE,
            status_line_height(),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    let chevron = status_chip_chevron_rect(&rect);
    push_down_chevron(commands, &chevron, clip, order + 3, style.text, opacity);
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    select_workbench_status_chip_style(node).text
}
