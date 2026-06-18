use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{component_variant_contains, first_non_empty};
use geometry::{
    chip_avatar_frame, chip_corner_radius, chip_delete_icon_frame, chip_frame, chip_icon_frame,
    chip_label_frame,
};
use style::{
    chip_avatar_background_color, chip_background_color, chip_border_color, chip_border_width,
    chip_delete_icon_color, chip_foreground_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod geometry;
mod style;

const CHIP_DELETE_STROKE: f32 = 2.0;
const CHIP_DELETE_DIAGONAL_DOT_COUNT: usize = 5;

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
    let frame = chip_avatar_frame(node, rect);
    let corner_radius = frame.height * 0.5;
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(chip_avatar_background_color(node)),
        None,
        0.0,
        corner_radius,
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
    let frame = chip_icon_frame(node, rect);
    let center_y = frame.y + frame.height * 0.5;
    let color = chip_foreground_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: frame.x,
            y: center_y - 1.0,
            width: frame.width,
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
            x: frame.x + frame.width * 0.5 - 1.0,
            y: frame.y,
            width: 2.0,
            height: frame.height,
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
    let Some((frame, font_size, line_height)) = chip_label_frame(node, rect, &label) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
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
    let frame = chip_delete_icon_frame(node, rect);
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
