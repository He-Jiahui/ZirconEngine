use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::is_command_palette;
use super::layout::{pixel_aligned_rect, row_rect};
use super::panel::{push_command_palette_empty_message, push_command_palette_panel_commands};
use super::rows::push_command_row_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_command_palette(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    push_command_palette_panel_commands(commands, node, &rect, clip, order, opacity);

    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        push_command_palette_empty_message(commands, &rect, clip, order + 3, opacity);
        return true;
    }

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        push_command_row_commands(
            commands,
            &option,
            &row_rect(&rect, row),
            clip,
            order + 4 + row as i32 * 3,
            opacity,
        );
    }

    true
}
