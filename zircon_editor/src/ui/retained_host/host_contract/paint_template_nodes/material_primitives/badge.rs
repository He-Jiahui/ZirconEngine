use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{component_variant_contains, first_non_empty};
use geometry::{
    badge_overlay_frame, badge_overlay_radius, badge_overlay_text_frame, badge_root_text_frame,
};
use style::{
    badge_overlay_background_color, badge_overlay_border_color, badge_overlay_border_width,
    badge_overlay_text_color, badge_root_background_color, badge_root_border_color,
    badge_root_border_width, badge_root_text_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod geometry;
mod style;

pub(super) fn push_badge_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_badge_slot_node(node) {
        return true;
    }
    if !is_badge_root_node(node) {
        return false;
    }

    push_badge_root_surface(commands, node, rect, clip, order, opacity);
    push_badge_root_label(commands, node, rect, clip, order + 1, opacity);
    push_badge_overlay(commands, node, rect, clip, order + 2, opacity);
    true
}

fn is_badge_root_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "badge" | "Badge" | "mui-badge" | "MuiBadge"
    ) || matches!(node.role.as_str(), "Badge" | "MuiBadge")
}

fn is_badge_slot_node(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "muiBadgeSlot")
        || component_variant_contains(node, "BadgeSlot")
        || component_variant_contains(node, "badgeSlot")
}

fn push_badge_root_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let background = badge_root_background_color(node);
    let border_width = badge_root_border_width(node);
    let border = badge_root_border_color(node, border_width);
    if background.is_none() && border.is_none() && border_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border,
        border_width.max(0.0),
        badge_root_corner_radius(node),
        opacity,
    ));
}

fn push_badge_root_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = badge_root_label(node);
    if label.is_empty() {
        return;
    }
    let text_frame = badge_root_text_frame(node, rect, &label);
    commands.push(HostPaintCommand::text(
        text_frame.rect,
        Some(clip.clone()),
        order,
        label,
        badge_root_text_color(node),
        text_frame.font_size,
        text_frame.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_badge_overlay(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if badge_is_invisible(node) {
        return;
    }
    let display = badge_display_text(node);
    let dot = badge_is_dot(node);
    if !dot && display.is_empty() {
        return;
    }
    let badge_rect = badge_overlay_frame(node, rect, &display, dot);
    if badge_rect.width <= 0.0 || badge_rect.height <= 0.0 {
        return;
    }
    let background = badge_overlay_background_color(node);
    let foreground = badge_overlay_text_color(node);
    let border_width = badge_overlay_border_width(node);
    commands.push(HostPaintCommand::quad(
        badge_rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        Some(badge_overlay_border_color(node, background)),
        border_width,
        badge_overlay_radius(dot),
        opacity,
    ));
    if !dot {
        push_badge_overlay_text(
            commands,
            &display,
            &badge_rect,
            clip,
            order + 1,
            foreground,
            opacity,
        );
    }
}

fn push_badge_overlay_text(
    commands: &mut Vec<HostPaintCommand>,
    display: &str,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let text_frame = badge_overlay_text_frame(display, rect);
    commands.push(HostPaintCommand::text(
        text_frame.rect,
        Some(clip.clone()),
        order,
        display.to_string(),
        color,
        text_frame.font_size,
        text_frame.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn badge_root_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[node.text.as_str(), node.options_text.as_str()])
        .trim()
        .to_string()
}

fn badge_display_text(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[node.value_text.as_str(), node.validation_message.as_str()])
        .trim()
        .to_string()
}

fn badge_is_dot(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "dot")
}

fn badge_is_invisible(node: &TemplatePaneNodeData) -> bool {
    node.disabled
        || component_variant_contains(node, "invisible")
        || component_variant_contains(node, "hidden")
}

fn badge_root_corner_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}
