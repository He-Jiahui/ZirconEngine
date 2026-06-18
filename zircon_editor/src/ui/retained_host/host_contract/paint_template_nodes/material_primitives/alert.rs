use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{component_variant_contains, first_non_empty};
use geometry::{
    alert_action_frame, alert_action_width, alert_icon_frame, alert_icon_mark_frame,
    alert_message_frame, alert_message_left, alert_message_right, alert_rect,
};
use style::{
    alert_action_color, alert_background_color, alert_border_color, alert_border_width,
    alert_corner_radius, alert_icon_color, alert_icon_cutout_color, alert_text_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod geometry;
mod style;

const ALERT_CLOSE_DOT_EDGE: f32 = 2.0;

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

    let paint_rect = alert_rect(rect);
    if paint_rect.width <= 0.0 || paint_rect.height <= 0.0 {
        return true;
    }

    push_alert_surface(commands, node, &paint_rect, clip, order, opacity);
    if alert_has_icon(node) {
        push_alert_icon(commands, node, &paint_rect, clip, order + 1, opacity);
    }
    let message_left = alert_message_left(node, &paint_rect);
    let message_right = alert_message_right(node, &paint_rect);
    let action_width = alert_action_width(node);
    push_alert_message(
        commands,
        node,
        &paint_rect,
        message_left,
        message_right,
        clip,
        order + 2,
        opacity,
    );
    if action_width > 0.0 {
        push_alert_action(commands, node, &paint_rect, clip, order + 3, opacity);
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
    let frame = alert_icon_frame(rect);
    let color = alert_icon_color(node);
    let mark = alert_icon_mark_frame(&frame);
    let mark_radius = mark.height * 0.5;
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        mark_radius,
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
    let Some((frame, font_size, line_height)) =
        alert_message_frame(node, rect, left, right, &message)
    else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
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
    let frame = alert_action_frame(rect);
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
