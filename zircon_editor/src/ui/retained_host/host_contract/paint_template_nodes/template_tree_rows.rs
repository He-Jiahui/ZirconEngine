use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_tree_row_style, WorkbenchTreeRowStyle};
#[cfg(test)]
#[path = "template_tree_rows_tests.rs"]
mod tests;
use super::template_node_labels::template_node_label;
use super::template_tree_row_geometry::{
    tree_action_rect, tree_disclosure_rect, tree_guide_x, tree_icon_rect, tree_label_rect,
    tree_line_height, TREE_FONT_SIZE, TREE_GUIDE_COLOR, TREE_ROW_RADIUS,
};
use super::template_tree_row_glyphs::{
    push_tree_disclosure_glyph, push_tree_eye_action_glyph, push_tree_kebab_action_glyph,
    push_tree_lock_action_glyph, push_tree_object_icon_glyph,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_tree_row_commands(
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
        tree_row_style(node).state,
        opacity,
    );
    push_tree_label(commands, node, rect, &icon, clip, order + 4, opacity);
    push_tree_actions(commands, node, rect, clip, order + 5, opacity);
    true
}

fn is_workbench_tree_row(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::TreeRow)
}

fn push_tree_row_surface(
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
}

fn push_tree_indent_guides(
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

fn push_tree_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let text_rect = tree_label_rect(rect, icon);
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        tree_text_color(node),
        TREE_FONT_SIZE,
        tree_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_tree_actions(
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

fn tree_row_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tree_row_style(node).background
}

fn tree_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tree_row_style(node).border
}

fn tree_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    tree_row_style(node).border_width
}

fn tree_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).text
}

fn tree_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).icon
}

fn tree_secondary_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).secondary
}

fn tree_action_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_row_style(node).action
}

fn tree_row_style(node: &TemplatePaneNodeData) -> WorkbenchTreeRowStyle {
    select_workbench_tree_row_style(node)
}

fn shows_tree_lock_action(node: &TemplatePaneNodeData) -> bool {
    let id = node.control_id.as_str();
    node.tree_depth <= 1
        || id.contains("Audio")
        || id.contains("Root")
        || id.contains("Environment")
}
