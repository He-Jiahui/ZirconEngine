use slint::{Model, ModelRc};

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::frame::HostRgbaFrame;
use super::geometry::{frame_from_template, inset, is_visible_frame, translated};
use super::render_commands::{draw_host_paint_commands, HostPaintCommand};

const PANEL: [u8; 4] = [32, 37, 46, 255];
const PANEL_INSET: [u8; 4] = [23, 28, 36, 255];
const PANEL_HOVERED: [u8; 4] = [39, 48, 62, 255];
const PANEL_DISABLED: [u8; 4] = [30, 33, 39, 255];
const BUTTON: [u8; 4] = [42, 50, 63, 255];
const BUTTON_PRIMARY: [u8; 4] = [60, 104, 176, 255];
const SELECTED: [u8; 4] = [54, 83, 130, 255];
const BORDER: [u8; 4] = [66, 76, 92, 255];
const ACCENT: [u8; 4] = [92, 156, 255, 255];
const TEXT: [u8; 4] = [210, 220, 235, 255];
const TEXT_MUTED: [u8; 4] = [133, 149, 170, 255];
const TEXT_DISABLED: [u8; 4] = [91, 99, 113, 255];

pub(super) fn draw_template_nodes(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
) -> bool {
    let mut commands = Vec::new();
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        push_template_node_commands(&mut commands, &node, origin, clip, row as i32);
    }
    draw_host_paint_commands(frame, &commands)
}

pub(super) fn has_template_nodes(nodes: &ModelRc<TemplatePaneNodeData>) -> bool {
    nodes.row_count() > 0
}

fn push_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
) {
    let local = frame_from_template(&node.frame);
    let rect = translated(&local, origin.x, origin.y);
    if !is_visible_frame(&rect) {
        return;
    }

    if draws_surface(node) {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(surface_color(node)),
            draws_border(node).then_some(border_color(node)),
            node.border_width.max(0.0),
            1.0,
        ));
    }

    let label = node_label(node);
    if !label.is_empty() || node.role.as_str() == "Label" || node.role.as_str() == "Button" {
        let text_rect = inset(&rect, 5.0);
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_rect.x,
                y: text_rect.y + (text_rect.height * 0.5).min(10.0).max(3.0),
                width: text_rect.width,
                height: 3.0,
            },
            Some(clip.clone()),
            order,
            label,
            text_color(node),
            1.0,
        ));
    }
}

fn draws_surface(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Panel" | "Button" | "Mount")
        || !node.surface_variant.is_empty()
        || !node.button_variant.is_empty()
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.hovered
        || node.pressed
        || node.focused
        || node.disabled
}

fn draws_border(node: &TemplatePaneNodeData) -> bool {
    node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.focused
        || matches!(node.role.as_str(), "Button" | "Mount")
}

fn surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PANEL_DISABLED;
    }
    if node.selected || node.focused || node.pressed {
        return SELECTED;
    }
    if node.button_variant.as_str() == "primary" || node.surface_variant.as_str() == "accent" {
        return BUTTON_PRIMARY;
    }
    if node.hovered || node.drop_hovered || node.active_drag_target {
        return PANEL_HOVERED;
    }
    match node.role.as_str() {
        "Button" => BUTTON,
        _ if node.surface_variant.as_str() == "inset" => PANEL_INSET,
        _ => PANEL,
    }
}

fn border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.selected || node.focused || node.pressed {
        ACCENT
    } else {
        BORDER
    }
}

fn text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return TEXT_DISABLED;
    }
    match node.text_tone.as_str() {
        "muted" | "subtle" => TEXT_MUTED,
        _ => TEXT,
    }
}

fn node_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
        node.control_id.as_str(),
        node.node_id.as_str(),
    ])
    .to_string()
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
