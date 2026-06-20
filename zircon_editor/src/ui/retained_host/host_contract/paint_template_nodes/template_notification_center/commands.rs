use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::is_notification_center;
use super::layout::{pixel_aligned_rect, row_rect};
use super::panel::{push_empty_notification_message, push_notification_panel_commands};
use super::row::push_notification_row;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_center_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_notification_center(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    push_notification_panel_commands(commands, node, &rect, clip, order, opacity);

    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        push_empty_notification_message(commands, node, &rect, clip, order + 2, opacity);
        return true;
    }

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        push_notification_row(
            commands,
            &option,
            &row_rect(&rect, row),
            clip,
            order + 3 + row as i32 * 4,
            opacity,
        );
    }

    true
}
