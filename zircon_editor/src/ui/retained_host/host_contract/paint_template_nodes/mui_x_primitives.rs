use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod charts;
mod chat;
mod data_grid;
mod kind;
mod pickers;
mod shared;
mod tree_view;

use self::kind::{MuiXKind, mui_x_kind};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shared::{
    component_variant_contains, matches_any_role, node_background, node_radius, push_quad,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_mui_x_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match mui_x_kind(node) {
        Some(MuiXKind::TreeView) => {
            tree_view::push_tree_view(commands, node, rect, clip, order, opacity)
        }
        Some(MuiXKind::DataGrid) => {
            data_grid::push_data_grid(commands, node, rect, clip, order, opacity)
        }
        Some(MuiXKind::DateTimePickers) => {
            pickers::push_date_time_picker(commands, node, rect, clip, order, opacity)
        }
        Some(MuiXKind::Chart(kind)) => {
            charts::push_chart(commands, node, rect, clip, order, opacity, kind)
        }
        Some(MuiXKind::Chat(kind)) => {
            chat::push_chat(commands, node, rect, clip, order, opacity, kind)
        }
        None => return false,
    }
    true
}
