use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{METRICS, PALETTE};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tree_row_geometry::{tree_action_button_rect, tree_action_icon_rect};
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
    let eye_button = tree_action_button_rect(rect, 1);
    let eye = tree_action_icon_rect(&eye_button);
    push_tree_action_button_slot(commands, &eye_button, clip, order, opacity);
    push_tree_eye_action_glyph(
        commands,
        &eye,
        clip,
        order + 1,
        tree_action_color(node),
        opacity,
    );

    let secondary_button = tree_action_button_rect(rect, 0);
    let secondary = tree_action_icon_rect(&secondary_button);
    if node.selected || node.checked {
        push_tree_action_button_slot(commands, &secondary_button, clip, order + 2, opacity);
        push_tree_kebab_action_glyph(
            commands,
            &secondary,
            clip,
            order + 3,
            tree_action_color(node),
            opacity,
        );
    } else if shows_tree_lock_action(node) {
        push_tree_action_button_slot(commands, &secondary_button, clip, order + 2, opacity);
        push_tree_lock_action_glyph(
            commands,
            &secondary,
            clip,
            order + 3,
            tree_action_color(node),
            opacity,
        );
    }
}

fn push_tree_action_button_slot(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_hover),
        Some(PALETTE.border),
        METRICS.border_width,
        METRICS.radius_control,
        opacity,
    ));
}

fn shows_tree_lock_action(node: &TemplatePaneNodeData) -> bool {
    let id = node.control_id.as_str();
    node.tree_depth <= 1
        || id.contains("Audio")
        || id.contains("Root")
        || id.contains("Environment")
}
