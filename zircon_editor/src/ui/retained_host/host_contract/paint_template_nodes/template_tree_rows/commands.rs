use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{tree_disclosure_rect, tree_icon_rect};
use super::super::template_tree_row_glyphs::{
    push_tree_disclosure_glyph, push_tree_object_icon_glyph,
};
use super::actions::push_tree_actions;
use super::geometry::{
    has_paintable_tree_row_extent, tree_row_contains, tree_row_has_action_space,
};
use super::identity::is_workbench_tree_row;
use super::labels::push_tree_label;
use super::layers::{
    action_slot_order, disclosure_order, indent_guides_order, label_order, object_icon_order,
};
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
    if !has_paintable_tree_row_extent(rect) {
        return true;
    }

    push_tree_row_surface(commands, node, rect, clip, order, opacity);
    push_tree_indent_guides(
        commands,
        node,
        rect,
        clip,
        indent_guides_order(order),
        opacity,
    );

    let disclosure = tree_disclosure_rect(node, rect);
    if !tree_row_contains(rect, &disclosure) {
        return true;
    }
    push_tree_disclosure_glyph(
        commands,
        node,
        &disclosure,
        clip,
        disclosure_order(order),
        tree_secondary_color(node),
        opacity,
    );

    let icon = tree_icon_rect(&disclosure);
    if !tree_row_contains(rect, &icon) {
        return true;
    }
    push_tree_object_icon_glyph(
        commands,
        node,
        &icon,
        clip,
        object_icon_order(order),
        tree_icon_color(node),
        tree_row_state(node),
        opacity,
    );
    push_tree_label(
        commands,
        node,
        rect,
        &icon,
        clip,
        label_order(order),
        opacity,
    );
    if tree_row_has_action_space(rect) {
        push_tree_actions(
            commands,
            node,
            rect,
            clip,
            action_slot_order(order),
            opacity,
        );
    }
    true
}
