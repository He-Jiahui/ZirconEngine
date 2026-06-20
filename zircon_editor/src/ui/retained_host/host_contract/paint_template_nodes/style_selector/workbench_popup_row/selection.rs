use super::model::{WorkbenchPopupRowState, WorkbenchPopupRowStyle};
use super::palette::WORKBENCH_POPUP_ROW_DANGER_TEXT;
use super::state::{is_hot, is_unavailable};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_popup_row_style(
    row: WorkbenchPopupRowState,
) -> WorkbenchPopupRowStyle {
    let state = row
        .painter_state()
        .resolved_state_for_family(UiPainterFamily::PopupRow);
    let marked = row.marked();
    let hot = is_hot(state);

    WorkbenchPopupRowStyle {
        background: popup_row_background(state, marked, hot),
        selection_mark: popup_row_selection_mark(state, marked),
        text: popup_row_text_color(row, state, marked, hot),
        shortcut: popup_row_shortcut_color(state),
        adornment: popup_row_adornment_color(row, state, marked, hot),
        state,
    }
}

fn popup_row_background(state: UiPainterResolvedState, marked: bool, hot: bool) -> Option<[u8; 4]> {
    if is_unavailable(state) {
        None
    } else if marked {
        Some(PALETTE.surface_selected)
    } else if hot {
        Some(PALETTE.surface_hover)
    } else {
        None
    }
}

fn popup_row_selection_mark(state: UiPainterResolvedState, marked: bool) -> Option<[u8; 4]> {
    (marked && !is_unavailable(state)).then_some(PALETTE.focus_ring)
}

fn popup_row_text_color(
    row: WorkbenchPopupRowState,
    state: UiPainterResolvedState,
    marked: bool,
    hot: bool,
) -> [u8; 4] {
    if is_unavailable(state) {
        PALETTE.text_disabled
    } else if row.danger {
        WORKBENCH_POPUP_ROW_DANGER_TEXT
    } else if marked || hot {
        PALETTE.focus_ring
    } else {
        PALETTE.text
    }
}

fn popup_row_shortcut_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text_muted
    }
}

fn popup_row_adornment_color(
    row: WorkbenchPopupRowState,
    state: UiPainterResolvedState,
    marked: bool,
    hot: bool,
) -> [u8; 4] {
    if is_unavailable(state) {
        PALETTE.text_disabled
    } else if row.danger {
        WORKBENCH_POPUP_ROW_DANGER_TEXT
    } else if marked || hot {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}
