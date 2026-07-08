use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{
    tree_guide_color, tree_guide_opacity, tree_guide_rect, tree_row_radius,
};
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
