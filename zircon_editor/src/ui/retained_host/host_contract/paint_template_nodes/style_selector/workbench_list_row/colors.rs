use super::palette::{WorkbenchListRowPalette, workbench_list_row_palette};
use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    list_row_text_color_from_palette(node, state, marked, workbench_list_row_palette())
}

fn list_row_text_color_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        palette.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    list_row_adornment_color_from_palette(node, state, marked, workbench_list_row_palette())
}

fn list_row_adornment_color_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        palette.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if marked {
        palette.marked_adornment
    } else {
        palette.text_muted
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
