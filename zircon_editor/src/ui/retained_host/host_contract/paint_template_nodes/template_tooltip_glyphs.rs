use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

const TOOLTIP_ARROW_SIZE: f32 = 8.0;
const TOOLTIP_ICON_SIZE: f32 = 18.0;

pub(super) fn tooltip_arrow_size(node: &TemplatePaneNodeData) -> f32 {
    let size = if node.value_number > 0.0 {
        node.value_number
    } else {
        TOOLTIP_ARROW_SIZE
    };
    size.clamp(4.0, 14.0)
}

pub(super) fn tooltip_icon_size(node: &TemplatePaneNodeData) -> f32 {
    let size = if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        TOOLTIP_ICON_SIZE
    };
    size.clamp(10.0, 24.0)
}

pub(super) fn push_tooltip_arrow(
    commands: &mut Vec<HostPaintCommand>,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    arrow_size: f32,
    fill: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let size = arrow_size.round().max(4.0) as u32;
    let x = bubble.x + bubble.width * 0.5 - size as f32 * 0.5;
    let y = bubble.y + bubble.height - 1.0;
    push_diamond(commands, x, y, size, clip, order, border, opacity);

    let fill_size = size.saturating_sub(2).max(2);
    push_diamond(
        commands,
        x + 1.0,
        y + 1.0,
        fill_size,
        clip,
        order + 1,
        fill,
        opacity,
    );
}

fn push_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    size: u32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let size = size.max(2);
    let center = (size as f32 - 1.0) * 0.5;
    for row in 0..size {
        let distance = (row as f32 - center).abs();
        let row_width = (size as f32 - distance * 2.0).ceil().max(1.0);
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x + (size as f32 - row_width) * 0.5,
                y: y + row as f32,
                width: row_width,
                height: 1.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

pub(super) fn push_tooltip_info_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon_size: f32,
    color: [u8; 4],
    opacity: f32,
) {
    let y = if node.layout_content_offset_y > 0.0 {
        rect.y + node.layout_content_offset_y
    } else {
        rect.y + rect.height - icon_size
    };
    let icon = FrameRect {
        x: rect.x + (rect.width - icon_size).max(0.0) * 0.5,
        y,
        width: icon_size,
        height: icon_size,
    };
    commands.push(HostPaintCommand::quad(
        icon.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        icon_size * 0.5,
        opacity,
    ));

    let stem_width = (icon_size * 0.12).max(2.0);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.45,
            width: stem_width,
            height: icon.height * 0.33,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.25,
            width: stem_width,
            height: stem_width,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        stem_width * 0.5,
        opacity,
    ));
}
