use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchStatusSignalStyle;

use super::super::geometry::{warning_mark_segments, STATUS_ITEM_ICON_SIZE};
use super::super::segments::push_segments;

pub(super) fn push_warning_signal(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchStatusSignalStyle,
    mark_width: f32,
    opacity: f32,
) {
    push_warning_triangle(commands, rect, clip, order, style.icon_fill, opacity);
    push_segments(
        commands,
        clip,
        order + 1,
        style.mark,
        opacity,
        &warning_mark_segments(rect, mark_width),
    );
}

fn push_warning_triangle(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let center_x = rect.x + rect.width * 0.5;
    let scale_x = rect.width / STATUS_ITEM_ICON_SIZE;
    let scale_y = rect.height / STATUS_ITEM_ICON_SIZE;
    for (row, width) in [2.0, 4.0, 6.0, 8.0, 10.0, 12.0].into_iter().enumerate() {
        let width = width * scale_x;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: center_x - width * 0.5,
                y: rect.y + (2.0 + row as f32 * 1.7) * scale_y,
                width,
                height: 2.0 * scale_y,
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
