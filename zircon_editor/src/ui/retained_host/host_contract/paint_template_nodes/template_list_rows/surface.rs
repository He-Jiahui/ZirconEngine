use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::render_commands::HostPaintCommand;
use super::super::template_row_metrics::ROW_SURFACE_RADIUS;
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
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        list_row_border(node),
        list_row_border_width(node),
        ROW_SURFACE_RADIUS,
        opacity,
    ));
    if is_selected_row(node) {
        commands.push(HostPaintCommand::quad(
            selection_indicator_rect(rect),
            Some(clip.clone()),
            order + 1,
            Some(PALETTE.accent),
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

fn selection_indicator_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: METRICS.selection_indicator_width.min(rect.width).max(1.0),
        height: rect.height,
    }
}
