use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::cells::{table_content_offset, TABLE_ACTION_WIDTH};
use super::identity::is_table_header;
use super::style::table_row_style;

pub(super) fn push_table_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let action_rect = FrameRect {
        x: rect.x + rect.width - TABLE_ACTION_WIDTH + 7.0 + content_offset_x,
        y: rect.y + (rect.height - 14.0).max(0.0) * 0.5 + content_offset_y,
        width: 14.0,
        height: 14.0,
    };
    let action_color = table_row_style(node).action;
    if is_table_header(node) {
        push_table_gear(commands, &action_rect, clip, order, action_color, opacity);
    } else {
        push_table_kebab(commands, &action_rect, clip, order, action_color, opacity);
    }
}

fn push_table_kebab(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for y in [3.0, 6.0, 9.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + y,
                width: 2.0,
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

fn push_table_gear(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 2.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 11.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 11.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 6.0,
            y: rect.y + 6.0,
            width: 2.0,
            height: 2.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
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
