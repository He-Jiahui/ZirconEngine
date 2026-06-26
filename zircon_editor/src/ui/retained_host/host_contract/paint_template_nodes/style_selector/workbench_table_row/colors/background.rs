use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::palette::{
    WORKBENCH_TABLE_HEADER_BG, WORKBENCH_TABLE_HOVER_BG, WORKBENCH_TABLE_ROW_BG,
    WORKBENCH_TABLE_SELECTED_BG, WORKBENCH_TABLE_TAIL_BG,
};
use super::super::state::{is_hot, is_unavailable_table_row_state};
use super::declared::declared_background_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    header: bool,
    tail: bool,
) -> [u8; 4] {
    if is_unavailable_table_row_state(state) {
        PALETTE.surface_disabled
    } else if marked {
        WORKBENCH_TABLE_SELECTED_BG
    } else if state == UiPainterResolvedState::Pressed {
        PALETTE.surface_pressed
    } else if is_hot(state) {
        WORKBENCH_TABLE_HOVER_BG
    } else if header {
        WORKBENCH_TABLE_HEADER_BG
    } else {
        declared_background_color(node).unwrap_or_else(|| {
            if tail {
                WORKBENCH_TABLE_TAIL_BG
            } else {
                WORKBENCH_TABLE_ROW_BG
            }
        })
    }
}
