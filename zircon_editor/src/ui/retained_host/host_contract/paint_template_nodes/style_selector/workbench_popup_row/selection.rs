use super::super::workbench_row_selection::selected_row_outline_color;
use super::model::{WorkbenchPopupRowState, WorkbenchPopupRowStyle};
use super::palette::workbench_popup_row_palette;
use super::state::{is_hot, is_unavailable};
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_popup_row_style(
    row: WorkbenchPopupRowState,
) -> WorkbenchPopupRowStyle {
    let state = row
        .painter_state()
        .resolved_state_for_family(UiPainterFamily::PopupRow);
    let marked = row.marked();
    let hot = row.hot() || is_hot(state);

    WorkbenchPopupRowStyle {
        background: popup_row_background(state, marked, hot),
        outline: popup_row_outline(state, marked),
        text: popup_row_text_color(row, state),
        shortcut: popup_row_shortcut_color(state),
        adornment: popup_row_adornment_color(row, state, marked, hot),
        state,
    }
}

fn popup_row_background(state: UiPainterResolvedState, marked: bool, hot: bool) -> Option<[u8; 4]> {
    let palette = workbench_popup_row_palette();
    if is_unavailable(state) {
        None
    } else if marked {
        Some(palette.marked_background)
    } else if hot {
        Some(palette.hot_background)
    } else {
        None
    }
}

fn popup_row_outline(state: UiPainterResolvedState, marked: bool) -> Option<[u8; 4]> {
    ((marked || state == UiPainterResolvedState::Focused) && !is_unavailable(state))
        .then_some(selected_row_outline_color())
}

fn popup_row_text_color(row: WorkbenchPopupRowState, state: UiPainterResolvedState) -> [u8; 4] {
    let palette = workbench_popup_row_palette();
    if is_unavailable(state) {
        palette.text_disabled
    } else if row.danger {
        palette.danger_text
    } else {
        palette.text
    }
}

fn popup_row_shortcut_color(state: UiPainterResolvedState) -> [u8; 4] {
    let palette = workbench_popup_row_palette();
    if is_unavailable(state) {
        palette.text_disabled
    } else {
        palette.text_muted
    }
}

fn popup_row_adornment_color(
    row: WorkbenchPopupRowState,
    state: UiPainterResolvedState,
    marked: bool,
    hot: bool,
) -> [u8; 4] {
    let palette = workbench_popup_row_palette();
    if is_unavailable(state) {
        palette.text_disabled
    } else if row.danger {
        palette.danger_text
    } else if marked || hot {
        palette.text
    } else {
        palette.text_muted
    }
}
