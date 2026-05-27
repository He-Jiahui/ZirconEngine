use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, first_non_empty, resolved_style_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const ALERT_PADDING_X: f32 = 16.0;
const ALERT_ICON_EDGE: f32 = 22.0;
const ALERT_ICON_GAP: f32 = 12.0;
const ALERT_ACTION_EDGE: f32 = 20.0;
const ALERT_ACTION_GAP: f32 = 16.0;
const ALERT_ACTION_TRAILING: f32 = 8.0;
const ALERT_FONT_SIZE: f32 = 13.0;
const ALERT_TEXT_WIDTH_RATIO: f32 = 0.56;
const ALERT_DEFAULT_RADIUS: f32 = 4.0;
const ALERT_ICON_MARK_EDGE: f32 = 14.0;
const ALERT_CLOSE_DOT_EDGE: f32 = 2.0;

const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

pub(super) fn push_alert_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_alert_slot_node(node) {
        return true;
    }
    if !is_alert_root_node(node) {
        return false;
    }

    let alert_rect = pixel_aligned_rect(rect);
    if alert_rect.width <= 0.0 || alert_rect.height <= 0.0 {
        return true;
    }

    push_alert_surface(commands, node, &alert_rect, clip, order, opacity);
    let message_left = if alert_has_icon(node) {
        push_alert_icon(commands, node, &alert_rect, clip, order + 1, opacity);
        alert_rect.x + ALERT_PADDING_X + ALERT_ICON_EDGE + ALERT_ICON_GAP
    } else {
        alert_rect.x + ALERT_PADDING_X
    };
    let action_width = alert_action_width(node);
    let message_right = alert_rect.x + alert_rect.width - ALERT_PADDING_X - action_width;
    push_alert_message(
        commands,
        node,
        &alert_rect,
        message_left,
        message_right,
        clip,
        order + 2,
        opacity,
    );
    if action_width > 0.0 {
        push_alert_action(commands, node, &alert_rect, clip, order + 3, opacity);
    }

    true
}

fn is_alert_root_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "alert" | "Alert" | "mui-alert" | "MuiAlert"
    ) || matches!(node.role.as_str(), "Alert" | "MuiAlert")
}

fn is_alert_slot_node(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "muiAlertSlot")
        || component_variant_contains(node, "AlertSlot")
        || component_variant_contains(node, "alertSlot")
        || component_variant_token_starts_with(node, "alertSlot")
}

fn component_variant_token_starts_with(node: &TemplatePaneNodeData, expected_prefix: &str) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| {
            part.get(..expected_prefix.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected_prefix))
        })
}

fn push_alert_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        alert_background_color(node),
        alert_border_color(node),
        alert_border_width(node),
        alert_corner_radius(node, rect),
        opacity,
    ));
}

fn push_alert_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = FrameRect {
        x: rect.x + ALERT_PADDING_X,
        y: rect.y + (rect.height - ALERT_ICON_EDGE).max(0.0) * 0.5,
        width: ALERT_ICON_EDGE,
        height: ALERT_ICON_EDGE,
    };
    let color = alert_icon_color(node);
    let mark = FrameRect {
        x: frame.x + (frame.width - ALERT_ICON_MARK_EDGE) * 0.5,
        y: frame.y + (frame.height - ALERT_ICON_MARK_EDGE) * 0.5,
        width: ALERT_ICON_MARK_EDGE,
        height: ALERT_ICON_MARK_EDGE,
    };
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        ALERT_ICON_MARK_EDGE * 0.5,
        opacity,
    ));

    let center_x = mark.x + mark.width * 0.5;
    let center_y = mark.y + mark.height * 0.5;
    let cutout = alert_icon_cutout_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - 1.0,
            y: center_y - 4.0,
            width: 2.0,
            height: 6.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(cutout),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - 1.0,
            y: center_y + 4.0,
            width: 2.0,
            height: 2.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(cutout),
        None,
        0.0,
        1.0,
        opacity,
    ));
}

fn push_alert_message(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    left: f32,
    right: f32,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let message = alert_message(node);
    if message.is_empty() || right <= left {
        return;
    }
    let font_size = alert_font_size(node);
    let line_height = font_size * 1.45;
    let width = (message.chars().count() as f32 * font_size * ALERT_TEXT_WIDTH_RATIO)
        .min(right - left)
        .max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        message,
        alert_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_alert_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let edge = ALERT_ACTION_EDGE.min(rect.height - 8.0).max(1.0);
    let frame = FrameRect {
        x: rect.x + rect.width - ALERT_ACTION_TRAILING - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    };
    let color = alert_action_color(node);
    if alert_has_close_action(node) {
        push_alert_close_mark(commands, &frame, clip, order, color, opacity);
    } else {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: frame.x + 3.0,
                y: frame.y + frame.height * 0.5 - 1.0,
                width: frame.width - 6.0,
                height: 2.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_alert_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    frame: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let start_x = frame.x + frame.width * 0.28;
    let end_x = frame.x + frame.width * 0.72;
    let start_y = frame.y + frame.height * 0.28;
    let end_y = frame.y + frame.height * 0.72;
    for index in 0..5 {
        let ratio = index as f32 / 4.0;
        push_alert_close_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            start_y + (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
        );
        push_alert_close_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            end_y - (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
        );
    }
}

fn push_alert_close_dot(
    commands: &mut Vec<HostPaintCommand>,
    center_x: f32,
    center_y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - ALERT_CLOSE_DOT_EDGE * 0.5,
            y: center_y - ALERT_CLOSE_DOT_EDGE * 0.5,
            width: ALERT_CLOSE_DOT_EDGE,
            height: ALERT_CLOSE_DOT_EDGE,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        ALERT_CLOSE_DOT_EDGE * 0.5,
        opacity,
    ));
}

fn alert_message(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.validation_message.as_str(),
        node.options_text.as_str(),
    ])
    .to_string()
}

fn alert_action_width(node: &TemplatePaneNodeData) -> f32 {
    if alert_has_action(node) {
        ALERT_ACTION_EDGE + ALERT_ACTION_GAP
    } else {
        0.0
    }
}

fn alert_has_icon(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "hasIcon") || component_variant_contains(node, "alertSlotIcon")
}

fn alert_has_action(node: &TemplatePaneNodeData) -> bool {
    alert_has_close_action(node)
        || component_variant_contains(node, "hasAction")
        || component_variant_contains(node, "alertSlotAction")
}

fn alert_has_close_action(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "hasCloseAction")
        || component_variant_contains(node, "alertSlotCloseButton")
        || component_variant_contains(node, "alertSlotCloseIcon")
}

fn alert_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        ALERT_FONT_SIZE
    }
}

fn alert_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if alert_is_outlined(node) {
            None
        } else if alert_is_filled(node) {
            Some(alert_main_color(node))
        } else {
            Some(alert_container_color(node))
        }
    })
}

fn alert_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        (alert_border_width(node) > 0.0).then(|| {
            if alert_is_outlined(node) {
                alert_main_color(node)
            } else {
                PALETTE.border
            }
        })
    })
}

fn alert_border_width(node: &TemplatePaneNodeData) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if alert_is_outlined(node) {
        configured.max(1.0)
    } else {
        configured
    }
}

fn alert_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        ALERT_DEFAULT_RADIUS
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}

fn alert_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        if alert_is_filled(node) {
            alert_filled_text_color(node)
        } else {
            alert_main_color(node)
        }
    })
}

fn alert_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    if alert_is_filled(node) {
        alert_filled_text_color(node)
    } else {
        alert_main_color(node)
    }
}

fn alert_action_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    alert_text_color(node)
}

fn alert_icon_cutout_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if alert_is_filled(node) {
        alert_main_color(node)
    } else if alert_is_outlined(node) {
        [0, 0, 0, 0]
    } else {
        alert_container_color(node)
    }
}

fn alert_filled_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if alert_color_token(node) == "warning" {
        MUI_ON_WARNING
    } else {
        MUI_ON_DARK
    }
}

fn alert_main_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => MUI_SUCCESS_MAIN,
        "info" => MUI_INFO_MAIN,
        "error" | "danger" => MUI_ERROR_MAIN,
        "warning" => MUI_WARNING_MAIN,
        _ => PALETTE.info,
    }
}

fn alert_container_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => PALETTE.success_container,
        "info" => PALETTE.info_container,
        "error" | "danger" => PALETTE.error_container,
        "warning" => PALETTE.warning_container,
        _ => PALETTE.info_container,
    }
}

fn alert_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in ["success", "info", "warning", "error", "danger"] {
        if component_variant_contains(node, token)
            || component_variant_contains(node, &format!("color{}", pascal_case(token)))
        {
            return token;
        }
    }
    match first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]) {
        "success" => "success",
        "info" => "info",
        "warning" => "warning",
        "error" | "danger" => "error",
        _ => "success",
    }
}

fn alert_is_filled(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "filled")
}

fn alert_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
