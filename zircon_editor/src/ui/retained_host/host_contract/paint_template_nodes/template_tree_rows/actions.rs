use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::tree_action_rect;
use super::super::template_tree_row_glyphs::{
    push_tree_eye_action_glyph, push_tree_kebab_action_glyph, push_tree_lock_action_glyph,
};
use super::style::tree_action_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_actions(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let eye = tree_action_rect(rect, 1);
    push_tree_eye_action_glyph(
        commands,
        &eye,
        clip,
        order,
        tree_action_color(node),
        opacity,
    );

    let secondary = tree_action_rect(rect, 0);
    if node.selected || node.checked {
        push_tree_kebab_action_glyph(
            commands,
            &secondary,
            clip,
            order + 1,
            tree_action_color(node),
            opacity,
        );
    } else if shows_tree_lock_action(node) {
        push_tree_lock_action_glyph(
            commands,
            &secondary,
            clip,
            order + 1,
            tree_action_color(node),
            opacity,
        );
    }
}

fn shows_tree_lock_action(node: &TemplatePaneNodeData) -> bool {
    let id = node.control_id.as_str();
    node.tree_depth <= 1
        || id.contains("Audio")
        || id.contains("Root")
        || id.contains("Environment")
}
