use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, first_non_empty, resolved_style_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const BADGE_STANDARD_HEIGHT: f32 = 20.0;
const BADGE_STANDARD_MIN_WIDTH: f32 = 20.0;
const BADGE_STANDARD_PADDING_X: f32 = 6.0;
const BADGE_DOT_EDGE: f32 = 8.0;
const BADGE_STANDARD_RADIUS: f32 = 10.0;
const BADGE_DOT_RADIUS: f32 = 4.0;
const BADGE_FONT_SIZE: f32 = 12.0;
const BADGE_ROOT_FONT_SIZE: f32 = 12.0;
const BADGE_ROOT_TEXT_INSET_X: f32 = 8.0;
const BADGE_ROOT_TEXT_WIDTH_RATIO: f32 = 0.56;
const BADGE_CIRCULAR_OFFSET_RATIO: f32 = 0.14;
const BADGE_TEXT_WIDTH_RATIO: f32 = 0.56;
const MUI_PRIMARY_MAIN: [u8; 4] = [25, 118, 210, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_BADGE_DEFAULT_BG: [u8; 4] = [117, 117, 117, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

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
    let background = resolved_style_color(node.button_style.element.background_color.as_ref());
    let border = resolved_style_color(node.button_style.element.border_color.as_ref());
    let border_width = node
        .button_style
        .element
        .border_width
        .max(node.border_width);
    if background.is_none() && border.is_none() && border_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        background,
        border.or_else(|| (border_width > 0.0).then_some(PALETTE.border)),
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
    let font_size = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        BADGE_ROOT_FONT_SIZE
    };
    let line_height = font_size * 1.2;
    let available_width = (rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0).max(1.0);
    let text_width = (label.chars().count() as f32 * font_size * BADGE_ROOT_TEXT_WIDTH_RATIO)
        .min(available_width)
        .max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + BADGE_ROOT_TEXT_INSET_X,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        badge_root_text_color(node),
        font_size,
        line_height,
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
        if dot {
            BADGE_DOT_RADIUS
        } else {
            BADGE_STANDARD_RADIUS
        },
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
    let line_height = BADGE_FONT_SIZE;
    let text_width = (display.chars().count() as f32 * BADGE_FONT_SIZE * BADGE_TEXT_WIDTH_RATIO)
        .min(rect.width)
        .max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        display.to_string(),
        color,
        BADGE_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn badge_overlay_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    display: &str,
    dot: bool,
) -> FrameRect {
    let (width, height) = if dot {
        (BADGE_DOT_EDGE, BADGE_DOT_EDGE)
    } else {
        (
            (display.chars().count() as f32 * BADGE_FONT_SIZE * BADGE_TEXT_WIDTH_RATIO
                + BADGE_STANDARD_PADDING_X * 2.0)
                .max(BADGE_STANDARD_MIN_WIDTH),
            BADGE_STANDARD_HEIGHT,
        )
    };
    let (anchor_x, anchor_y) = badge_anchor_point(node, rect);
    FrameRect {
        x: (anchor_x - width * 0.5).round(),
        y: (anchor_y - height * 0.5).round(),
        width: width.round().max(1.0),
        height: height.round().max(1.0),
    }
}

fn badge_anchor_point(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let circular = component_variant_contains(node, "circular")
        || component_variant_contains(node, "overlapCircular");
    let offset_x = if circular {
        rect.width * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let offset_y = if circular {
        rect.height * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let left = badge_is_left_anchored(node);
    let bottom = badge_is_bottom_anchored(node);
    let x = if left {
        rect.x + offset_x
    } else {
        rect.x + rect.width - offset_x
    };
    let y = if bottom {
        rect.y + rect.height - offset_y
    } else {
        rect.y + offset_y
    };
    (x, y)
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

fn badge_is_left_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "left")
        || component_variant_contains(node, "anchorOriginTopLeft")
        || component_variant_contains(node, "anchorOriginBottomLeft")
}

fn badge_is_bottom_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "bottom")
        || component_variant_contains(node, "anchorOriginBottomLeft")
        || component_variant_contains(node, "anchorOriginBottomRight")
}

fn badge_root_corner_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

fn badge_root_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .unwrap_or(PALETTE.text)
}

fn badge_overlay_background_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match badge_color_token(node) {
        "primary" => MUI_PRIMARY_MAIN,
        "secondary" => MUI_SECONDARY_MAIN,
        "info" => MUI_INFO_MAIN,
        "success" => MUI_SUCCESS_MAIN,
        "warning" => MUI_WARNING_MAIN,
        "default" => MUI_BADGE_DEFAULT_BG,
        "error" | "danger" => MUI_ERROR_MAIN,
        _ => {
            if matches!(
                first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]),
                "error" | "danger"
            ) {
                MUI_ERROR_MAIN
            } else {
                MUI_BADGE_DEFAULT_BG
            }
        }
    }
}

fn badge_overlay_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match badge_color_token(node) {
        "warning" => MUI_ON_WARNING,
        "default" => MUI_ON_DARK,
        _ => MUI_ON_DARK,
    }
}

fn badge_overlay_border_color(node: &TemplatePaneNodeData, background: [u8; 4]) -> [u8; 4] {
    if component_variant_contains(node, "overlapCircular")
        || component_variant_contains(node, "circular")
    {
        background
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(background)
    }
}

fn badge_overlay_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.border_width
        .max(node.button_style.element.border_width)
        .max(0.0)
        .min(2.0)
}

fn badge_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in [
        "primary",
        "secondary",
        "error",
        "danger",
        "info",
        "success",
        "warning",
        "default",
    ] {
        if component_variant_contains(node, token) {
            return token;
        }
    }
    first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()])
}
