use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{tree_disclosure_rect, tree_icon_rect};
use super::super::template_tree_row_glyphs::{
    push_tree_disclosure_glyph, push_tree_object_icon_glyph,
};
use super::actions::push_tree_actions;
use super::identity::is_workbench_tree_row;
use super::labels::push_tree_label;
use super::style::{tree_icon_color, tree_row_state, tree_secondary_color};
use super::surface::{push_tree_indent_guides, push_tree_row_surface};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_tree_row(node) {
        return false;
    }

    push_tree_row_surface(commands, node, rect, clip, order, opacity);
    push_tree_indent_guides(commands, node, rect, clip, order + 1, opacity);

    let disclosure = tree_disclosure_rect(node, rect);
    push_tree_disclosure_glyph(
        commands,
        node,
        &disclosure,
        clip,
        order + 2,
        tree_secondary_color(node),
        opacity,
    );

    let icon = tree_icon_rect(&disclosure);
    push_tree_object_icon_glyph(
        commands,
        node,
        &icon,
        clip,
        order + 3,
        tree_icon_color(node),
        tree_row_state(node),
        opacity,
    );
    push_tree_label(commands, node, rect, &icon, clip, order + 4, opacity);
    push_tree_actions(commands, node, rect, clip, order + 5, opacity);
    true
}
