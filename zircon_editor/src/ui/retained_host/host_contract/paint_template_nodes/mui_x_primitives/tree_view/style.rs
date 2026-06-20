use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;

pub(super) fn tree_view_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.selected || node.checked || super::super::component_variant_contains(node, "multi") {
        PALETTE.success_container
    } else {
        PALETTE.surface_inset
    }
}

pub(super) fn tree_view_row_color(node: &TemplatePaneNodeData, row: i32) -> [u8; 4] {
    if row == 0 && (node.selected || node.checked) {
        PALETTE.surface_selected
    } else if row == 1 && (node.expanded || node.popup_open || node.focused) {
        PALETTE.surface_hover
    } else {
        PALETTE.surface
    }
}

pub(super) fn tree_view_marker_color(node: &TemplatePaneNodeData, row: i32) -> [u8; 4] {
    if row == 0 && (node.expanded || node.popup_open) {
        PALETTE.success
    } else {
        PALETTE.border
    }
}
