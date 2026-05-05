use slint::{Model, ModelRc};

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::frame::HostRgbaFrame;
use super::geometry::{frame_from_template, is_visible_frame, translated};
use super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::visual_assets::{slint_image_pixels, template_image_tint};

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
const DEFAULT_TEMPLATE_FONT_SIZE: f32 = 12.0;
const TEXT_HORIZONTAL_INSET: f32 = 5.0;
const TEXT_VERTICAL_INSET: f32 = 5.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;

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
            order * 4,
            Some(surface_color(node)),
            draws_border(node).then_some(border_color(node)),
            node.border_width.max(0.0),
            1.0,
        ));
    }

    push_template_image_command(commands, node, &rect, clip, order * 4 + 1);

    let label = node_label(node);
    if !label.is_empty() || node.role.as_str() == "Label" || node.role.as_str() == "Button" {
        let text_rect = text_rect_for_node(&rect);
        let font_size = node_font_size(node, text_rect.height);
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_rect.x,
                y: text_rect.y,
                width: text_rect.width,
                height: text_rect.height,
            },
            Some(clip.clone()),
            order * 4 + 2,
            label,
            text_color(node),
            font_size,
            font_size * 1.2,
            1.0,
        ));
    }
}

fn push_template_image_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
) {
    if !node.has_preview_image {
        return;
    }
    let Some(image) = slint_image_pixels(
        &node.preview_image,
        template_image_tint(is_icon_node(node)),
    ) else {
        return;
    };
    let image_rect = image_rect_for_node(node, rect);
    if !is_visible_frame(&image_rect) {
        return;
    }
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        image.width,
        image.height,
        image.rgba,
        1.0,
    ));
}

fn image_rect_for_node(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    if is_icon_node(node) {
        let inset = (rect.width.min(rect.height) * 0.16).min(4.0).max(0.0);
        let size = (rect.width.min(rect.height) - inset * 2.0).max(1.0);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    rect.clone()
}

fn is_icon_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon") || !node.icon_name.is_empty()
}

fn text_rect_for_node(rect: &FrameRect) -> FrameRect {
    let horizontal = TEXT_HORIZONTAL_INSET
        .min((rect.width * 0.25).max(0.0))
        .max(0.0);
    let vertical = TEXT_VERTICAL_INSET
        .min(((rect.height - MIN_TEXT_RECT_HEIGHT) * 0.5).max(1.0))
        .max(0.0);
    FrameRect {
        x: rect.x + horizontal,
        y: rect.y + vertical,
        width: (rect.width - horizontal * 2.0).max(0.0),
        height: (rect.height - vertical * 2.0).max(0.0),
    }
}

fn node_font_size(node: &TemplatePaneNodeData, available_height: f32) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DEFAULT_TEMPLATE_FONT_SIZE
    };
    requested.min(available_height.max(1.0)).max(1.0)
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
