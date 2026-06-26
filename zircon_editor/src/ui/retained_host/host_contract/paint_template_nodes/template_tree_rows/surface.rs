use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{tree_guide_x, TREE_GUIDE_COLOR, TREE_ROW_RADIUS};
use super::style::{tree_row_background, tree_row_border, tree_row_border_width};

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
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        tree_row_border(node),
        tree_row_border_width(node),
        TREE_ROW_RADIUS,
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_indent_guides(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let depth = node.tree_depth.max(0) as usize;
    for level in 0..depth {
        let guide_x = tree_guide_x(rect, level);
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: guide_x,
                y: rect.y - 1.0,
                width: 1.0,
                height: rect.height + 2.0,
            },
            Some(clip.clone()),
            order,
            Some(TREE_GUIDE_COLOR),
            None,
            0.0,
            0.0,
            opacity * 0.78,
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
