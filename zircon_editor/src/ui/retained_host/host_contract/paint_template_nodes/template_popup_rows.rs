use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod menu;
mod options;
mod surface;
mod text;

use menu::push_menu_row_commands;
use options::push_option_row_commands;

#[cfg(test)]
use super::template_popup_row_adornments::{
    menu_item_has_flag, menu_row_adornment_kind, PopupRowAdornmentKind,
};
#[cfg(test)]
use menu::popup_menu_row_style;
#[cfg(test)]
use options::popup_option_row_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_popup_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    bounds: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open {
        return;
    }
    if node.structured_menu_items.row_count() > 0 {
        push_menu_row_commands(commands, node, rect, clip, order, opacity);
    } else if node.structured_options.row_count() > 0 {
        push_option_row_commands(commands, node, rect, bounds, clip, order, opacity);
    }
}

#[cfg(test)]
#[path = "template_popup_rows_tests.rs"]
mod tests;
