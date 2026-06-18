use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::template_style_color::resolved_style_color;

mod charts;
mod chat;
mod data_grid;
mod pickers;
mod tree_view;

use self::{charts::ChartKind, chat::ChatKind};

pub(super) fn push_mui_x_primitive_commands(
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

enum MuiXKind {
    TreeView,
    DataGrid,
    DateTimePickers,
    Chart(ChartKind),
    Chat(ChatKind),
}

fn mui_x_kind(node: &TemplatePaneNodeData) -> Option<MuiXKind> {
    let component_role = node.component_role.as_str();
    let role = node.role.as_str();
    if tree_view::is_tree_view(component_role, role) {
        Some(MuiXKind::TreeView)
    } else if data_grid::is_data_grid(component_role, role) {
        Some(MuiXKind::DataGrid)
    } else if pickers::is_date_time_picker(component_role, role) {
        Some(MuiXKind::DateTimePickers)
    } else if let Some(kind) = charts::chart_kind(component_role, role) {
        Some(MuiXKind::Chart(kind))
    } else {
        chat::chat_kind(component_role, role).map(MuiXKind::Chat)
    }
}

fn matches_any_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| *candidate == component_role || *candidate == role)
}

fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    border_width: f32,
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(color),
        (border_width > 0.0).then_some(PALETTE.focus_ring),
        border_width,
        radius,
        opacity,
    ));
}

fn node_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

fn node_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

fn component_variant_contains(node: &TemplatePaneNodeData, expected: &str) -> bool {
    node.component_variant
        .split_whitespace()
        .any(|part| part == expected)
}
