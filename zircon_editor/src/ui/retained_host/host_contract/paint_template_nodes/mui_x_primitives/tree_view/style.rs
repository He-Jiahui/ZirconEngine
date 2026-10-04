use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

pub(super) fn tree_view_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    tree_view_surface_color_from_host(node, current_host_palette())
}

/// 行选中、悬停和宿主色板的优先级在此收敛，避免把整棵树的焦点扩散到每行。
pub(super) fn tree_view_row_color(node: &TemplatePaneNodeData, row: i32) -> [u8; 4] {
    tree_view_row_color_from_host(node, row, current_host_palette())
}

pub(super) fn tree_view_marker_color(node: &TemplatePaneNodeData, row: i32) -> [u8; 4] {
    tree_view_marker_color_from_host(node, row, current_host_palette())
}

fn tree_view_surface_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.selected || node.checked || super::super::component_variant_contains(node, "multi") {
        palette.success_container
    } else {
        palette.surface_inset
    }
}

fn tree_view_row_color_from_host(
    node: &TemplatePaneNodeData,
    row: i32,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if row == 0 && (node.selected || node.checked) {
        palette.surface_selected
    } else if row == 1 && (node.expanded || node.popup_open) {
        palette.surface_hover
    } else {
        palette.surface
    }
}

fn tree_view_marker_color_from_host(
    node: &TemplatePaneNodeData,
    row: i32,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if row == 0 && (node.expanded || node.popup_open) {
        palette.success
    } else {
        palette.border
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
