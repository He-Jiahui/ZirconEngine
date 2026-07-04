use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_row_metrics::{workbench_row_metrics, workbench_row_palette};
use super::style::{list_row_background, list_row_border, list_row_border_width};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = list_row_background(node) else {
        return;
    };
    let metrics = workbench_row_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        list_row_border(node),
        list_row_border_width(node),
        metrics.surface_radius,
        opacity,
    ));
    if is_selected_row(node) {
        let palette = workbench_row_palette();
        commands.push(HostPaintCommand::quad(
            selection_indicator_rect(rect, metrics.selection_indicator_width),
            Some(clip.clone()),
            order + 1,
            Some(palette.selection_indicator),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn is_selected_row(node: &TemplatePaneNodeData) -> bool {
    node.selected || node.checked
}

fn selection_indicator_rect(rect: &FrameRect, indicator_width: f32) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: indicator_width.min(rect.width).max(1.0),
        height: rect.height,
    }
}
