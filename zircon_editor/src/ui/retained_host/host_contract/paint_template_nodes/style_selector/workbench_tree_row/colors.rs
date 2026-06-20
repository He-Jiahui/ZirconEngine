use super::palette::{
    WORKBENCH_TREE_ROW_ACTION, WORKBENCH_TREE_ROW_ICON_MUTED, WORKBENCH_TREE_ROW_TEXT_NORMAL,
    WORKBENCH_TREE_ROW_TEXT_SELECTED,
};
use super::state::is_unavailable_tree_row_state;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_primary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_tree_row_state(state) {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_TEXT_NORMAL
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_icon_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_tree_row_state(state) {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_ICON_MUTED
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_secondary_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_tree_row_state(state) {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        PALETTE.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_action_color(
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_tree_row_state(state) {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_ACTION
    }
}
