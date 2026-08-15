use super::palette::{workbench_tree_row_palette, WorkbenchTreeRowPalette};
use super::state::is_unavailable_tree_row_state;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_primary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    tree_row_text_color_from_palette(state, marked, workbench_tree_row_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_icon_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    tree_row_text_color_from_palette(state, marked, workbench_tree_row_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_secondary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    tree_row_text_color_from_palette(state, marked, workbench_tree_row_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_action_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    tree_row_text_color_from_palette(state, marked, workbench_tree_row_palette())
}

fn tree_row_text_color_from_palette(
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchTreeRowPalette,
) -> [u8; 4] {
    if is_unavailable_tree_row_state(state) {
        palette.text_disabled
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}
