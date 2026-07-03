use super::state::is_unavailable_tree_row_state;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_primary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        palette.text_disabled
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_icon_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        palette.text_disabled
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_secondary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        palette.text_disabled
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_action_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        palette.text_disabled
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}
