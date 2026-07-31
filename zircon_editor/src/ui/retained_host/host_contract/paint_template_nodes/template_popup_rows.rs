use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod content;
mod geometry;
mod layers;
mod menu;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) mod metrics;
mod options;
mod surface;
mod text;

use geometry::{frame_is_within, has_paintable_popup_row_extent};
use menu::push_menu_row_commands;
use options::push_option_row_commands;

#[cfg(test)]
use super::template_popup_row_adornments::{
    PopupRowAdornmentKind, menu_item_has_flag, menu_row_adornment_kind,
};
#[cfg(test)]
use content::popup_row_content_style;
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
    if !node.popup_open
        || !has_paintable_popup_row_extent(rect)
        || !has_paintable_popup_row_extent(clip)
        || !frame_is_within(clip, rect)
    {
        return;
    }
    if node.structured_menu_items.row_count() > 0 {
        push_menu_row_commands(commands, node, rect, clip, order, opacity);
    } else if node.structured_options.row_count() > 0 {
        push_option_row_commands(commands, node, rect, bounds, clip, order, opacity);
    }
}

#[cfg(test)]
#[path = "template_popup_rows_tests/mod.rs"]
mod tests;
