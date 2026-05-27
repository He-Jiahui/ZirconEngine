use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, first_non_empty, resolved_style_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const CHIP_MEDIUM_HEIGHT: f32 = 32.0;
const CHIP_SMALL_HEIGHT: f32 = 24.0;
const CHIP_LABEL_FONT_SIZE: f32 = 13.0;
const CHIP_SMALL_LABEL_FONT_SIZE: f32 = 12.0;
const CHIP_LABEL_WIDTH_RATIO: f32 = 0.56;
const CHIP_LABEL_PADDING: f32 = 12.0;
const CHIP_LABEL_OUTLINED_PADDING: f32 = 11.0;
const CHIP_SMALL_LABEL_PADDING: f32 = 8.0;
const CHIP_SMALL_OUTLINED_LABEL_PADDING: f32 = 7.0;
const CHIP_AVATAR_MEDIUM_EDGE: f32 = 24.0;
const CHIP_AVATAR_SMALL_EDGE: f32 = 18.0;
const CHIP_DELETE_MEDIUM_EDGE: f32 = 22.0;
const CHIP_DELETE_SMALL_EDGE: f32 = 16.0;
const CHIP_DELETE_STROKE: f32 = 2.0;
const CHIP_DELETE_DIAGONAL_DOT_COUNT: usize = 5;
const MUI_PRIMARY_MAIN: [u8; 4] = [25, 118, 210, 255];
const MUI_PRIMARY_DARK: [u8; 4] = [21, 101, 192, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
const MUI_SECONDARY_DARK: [u8; 4] = [123, 31, 162, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_ERROR_DARK: [u8; 4] = [198, 40, 40, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_INFO_DARK: [u8; 4] = [1, 87, 155, 255];
const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_SUCCESS_DARK: [u8; 4] = [27, 94, 32, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_WARNING_DARK: [u8; 4] = [230, 81, 0, 255];
const MUI_CHIP_DEFAULT_FILLED: [u8; 4] = [66, 66, 66, 255];
const MUI_CHIP_DEFAULT_AVATAR: [u8; 4] = [117, 117, 117, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

pub(super) fn push_chip_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_chip_slot_node(node) {
        return true;
    }
    if !is_chip_root_node(node) {
        return false;
    }

    let chip_rect = chip_frame(node, rect);
    if chip_rect.width <= 0.0 || chip_rect.height <= 0.0 {
        return true;
    }

    push_chip_surface(commands, node, &chip_rect, clip, order, opacity);
    if chip_has_avatar(node) {
        push_chip_avatar(commands, node, &chip_rect, clip, order + 1, opacity);
    } else if chip_has_icon(node) {
        push_chip_icon(commands, node, &chip_rect, clip, order + 1, opacity);
    }
    push_chip_label(commands, node, &chip_rect, clip, order + 2, opacity);
    if chip_is_deletable(node) {
        push_chip_delete_icon(commands, node, &chip_rect, clip, order + 3, opacity);
    }

    true
}

fn is_chip_root_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "chip" | "Chip" | "mui-chip" | "MuiChip"
    ) || matches!(node.role.as_str(), "Chip" | "MuiChip")
}

fn is_chip_slot_node(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "muiChipSlot")
        || component_variant_contains(node, "ChipSlot")
        || component_variant_contains(node, "chipSlot")
        || component_variant_token_starts_with(node, "chipSlot")
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

fn chip_frame(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let target_height = chip_height(node).min(rect.height.max(1.0)).round();
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - target_height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(1.0),
        height: target_height.max(1.0),
    }
}

fn push_chip_surface(
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
        chip_background_color(node),
        chip_border_color(node),
        chip_border_width(node),
        chip_corner_radius(node, rect),
        opacity,
    ));
}

fn push_chip_avatar(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let edge = if chip_is_small(node) {
        CHIP_AVATAR_SMALL_EDGE
    } else {
        CHIP_AVATAR_MEDIUM_EDGE
    }
    .min(rect.height - 4.0)
    .max(1.0);
    let frame = FrameRect {
        x: rect.x + chip_leading_margin(node),
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    };
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(chip_avatar_background_color(node)),
        None,
        0.0,
        edge * 0.5,
        opacity,
    ));
}

fn push_chip_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let edge = if chip_is_small(node) {
        18.0_f32
    } else {
        20.0_f32
    }
    .min(rect.height - 4.0)
    .max(1.0);
    let x = rect.x + chip_leading_margin(node);
    let center_y = rect.y + rect.height * 0.5;
    let color = chip_foreground_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y: center_y - 1.0,
            width: edge,
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
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x + edge * 0.5 - 1.0,
            y: center_y - edge * 0.5,
            width: 2.0,
            height: edge,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
}

fn push_chip_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = chip_label(node);
    if label.is_empty() {
        return;
    }
    let font_size = chip_font_size(node);
    let line_height = font_size * 1.5;
    let left = rect.x + chip_label_left_padding(node);
    let right = rect.x + rect.width - chip_label_right_padding(node);
    if right <= left {
        return;
    }
    let estimated_width = label.chars().count() as f32 * font_size * CHIP_LABEL_WIDTH_RATIO;
    let width = estimated_width.min(right - left).max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        chip_foreground_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_chip_delete_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let edge = if chip_is_small(node) {
        CHIP_DELETE_SMALL_EDGE
    } else {
        CHIP_DELETE_MEDIUM_EDGE
    }
    .min(rect.height - 4.0)
    .max(1.0);
    let right_margin = if chip_is_small(node) { 4.0 } else { 5.0 };
    let frame = FrameRect {
        x: rect.x + rect.width - right_margin - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    };
    let color = chip_delete_icon_color(node);
    let start_x = frame.x + frame.width * 0.25;
    let end_x = frame.x + frame.width * 0.75;
    let start_y = frame.y + frame.height * 0.25;
    let end_y = frame.y + frame.height * 0.75;
    for index in 0..CHIP_DELETE_DIAGONAL_DOT_COUNT {
        let ratio = if CHIP_DELETE_DIAGONAL_DOT_COUNT <= 1 {
            0.0
        } else {
            index as f32 / (CHIP_DELETE_DIAGONAL_DOT_COUNT - 1) as f32
        };
        push_chip_delete_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            start_y + (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
        );
        push_chip_delete_dot(
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

fn push_chip_delete_dot(
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
            x: center_x - CHIP_DELETE_STROKE * 0.5,
            y: center_y - CHIP_DELETE_STROKE * 0.5,
            width: CHIP_DELETE_STROKE,
            height: CHIP_DELETE_STROKE,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        CHIP_DELETE_STROKE * 0.5,
        opacity,
    ));
}

fn chip_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[node.text.as_str(), node.value_text.as_str()]).to_string()
}

fn chip_height(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_SMALL_HEIGHT
    } else {
        CHIP_MEDIUM_HEIGHT
    }
}

fn chip_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured > 0.0 {
        configured.min(rect.height * 0.5)
    } else {
        rect.height * 0.5
    }
}

fn chip_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else if chip_is_small(node) {
        CHIP_SMALL_LABEL_FONT_SIZE
    } else {
        CHIP_LABEL_FONT_SIZE
    }
}

fn chip_label_left_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = if chip_is_small(node) {
        if chip_is_outlined(node) {
            CHIP_SMALL_OUTLINED_LABEL_PADDING
        } else {
            CHIP_SMALL_LABEL_PADDING
        }
    } else if chip_is_outlined(node) {
        CHIP_LABEL_OUTLINED_PADDING
    } else {
        CHIP_LABEL_PADDING
    };
    if chip_has_avatar(node) || chip_has_icon(node) {
        base + chip_leading_margin(node) + chip_leading_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_label_right_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = if chip_is_small(node) {
        if chip_is_outlined(node) {
            CHIP_SMALL_OUTLINED_LABEL_PADDING
        } else {
            CHIP_SMALL_LABEL_PADDING
        }
    } else if chip_is_outlined(node) {
        CHIP_LABEL_OUTLINED_PADDING
    } else {
        CHIP_LABEL_PADDING
    };
    if chip_is_deletable(node) {
        base + chip_delete_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_leading_edge(node: &TemplatePaneNodeData) -> f32 {
    if chip_has_avatar(node) {
        if chip_is_small(node) {
            CHIP_AVATAR_SMALL_EDGE
        } else {
            CHIP_AVATAR_MEDIUM_EDGE
        }
    } else if chip_has_icon(node) {
        if chip_is_small(node) {
            18.0
        } else {
            20.0
        }
    } else {
        0.0
    }
}

fn chip_delete_edge(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_DELETE_SMALL_EDGE
    } else {
        CHIP_DELETE_MEDIUM_EDGE
    }
}

fn chip_leading_margin(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        if chip_is_outlined(node) {
            2.0
        } else {
            4.0
        }
    } else if chip_is_outlined(node) {
        4.0
    } else {
        5.0
    }
}

fn chip_negative_slot_margin(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        4.0
    } else {
        6.0
    }
}

fn chip_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            None
        } else {
            Some(match chip_color_token(node) {
                "primary" => MUI_PRIMARY_MAIN,
                "secondary" => MUI_SECONDARY_MAIN,
                "error" => MUI_ERROR_MAIN,
                "info" => MUI_INFO_MAIN,
                "success" => MUI_SUCCESS_MAIN,
                "warning" => MUI_WARNING_MAIN,
                _ => MUI_CHIP_DEFAULT_FILLED,
            })
        }
    })
}

fn chip_foreground_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        let color = chip_color_token(node);
        if chip_is_outlined(node) {
            chip_palette_main(color).unwrap_or(PALETTE.text)
        } else if color == "warning" {
            MUI_ON_WARNING
        } else if color == "default" {
            PALETTE.text
        } else {
            MUI_ON_DARK
        }
    })
}

fn chip_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            Some(chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.border))
        } else if node.border_width > 0.0 || node.button_style.element.border_width > 0.0 {
            Some(PALETTE.border)
        } else {
            None
        }
    })
}

fn chip_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(if chip_is_outlined(node) { 1.0 } else { 0.0 })
}

fn chip_avatar_background_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match chip_color_token(node) {
        "primary" => MUI_PRIMARY_DARK,
        "secondary" => MUI_SECONDARY_DARK,
        "error" => MUI_ERROR_DARK,
        "info" => MUI_INFO_DARK,
        "success" => MUI_SUCCESS_DARK,
        "warning" => MUI_WARNING_DARK,
        _ => MUI_CHIP_DEFAULT_AVATAR,
    }
}

fn chip_delete_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if chip_is_outlined(node) {
        chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.text_muted)
    } else {
        chip_foreground_color(node)
    }
}

fn chip_palette_main(color: &str) -> Option<[u8; 4]> {
    match color {
        "primary" => Some(MUI_PRIMARY_MAIN),
        "secondary" => Some(MUI_SECONDARY_MAIN),
        "error" => Some(MUI_ERROR_MAIN),
        "info" => Some(MUI_INFO_MAIN),
        "success" => Some(MUI_SUCCESS_MAIN),
        "warning" => Some(MUI_WARNING_MAIN),
        _ => None,
    }
}

fn chip_color_token(node: &TemplatePaneNodeData) -> &str {
    if component_variant_contains(node, "primary")
        || component_variant_contains(node, "colorPrimary")
    {
        "primary"
    } else if component_variant_contains(node, "secondary")
        || component_variant_contains(node, "colorSecondary")
    {
        "secondary"
    } else if component_variant_contains(node, "error")
        || component_variant_contains(node, "colorError")
    {
        "error"
    } else if component_variant_contains(node, "info")
        || component_variant_contains(node, "colorInfo")
    {
        "info"
    } else if component_variant_contains(node, "success")
        || component_variant_contains(node, "colorSuccess")
    {
        "success"
    } else if component_variant_contains(node, "warning")
        || component_variant_contains(node, "colorWarning")
    {
        "warning"
    } else {
        "default"
    }
}

fn chip_is_small(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "small") || component_variant_contains(node, "sizeSmall")
}

fn chip_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
}

fn chip_is_deletable(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "deletable")
        || component_variant_contains(node, "hasDeleteIcon")
}

fn chip_has_avatar(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "hasAvatar")
        || component_variant_contains(node, "chipSlotAvatar")
}

fn chip_has_icon(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "hasIcon") || component_variant_contains(node, "chipSlotIcon")
}
