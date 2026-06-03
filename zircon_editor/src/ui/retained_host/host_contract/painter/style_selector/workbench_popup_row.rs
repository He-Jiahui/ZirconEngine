use super::super::theme::PALETTE;
use zircon_runtime_interface::ui::style::{
    UiPainterFamily, UiPainterResolvedState, UiPainterState,
};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_POPUP_ROW_DANGER_TEXT:
    [u8; 4] = [242, 95, 82, 255];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchPopupRowState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
    pub selected: bool,
    pub danger: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchPopupRowStyle {
    pub background: Option<[u8; 4]>,
    pub selection_mark: Option<[u8; 4]>,
    pub text: [u8; 4],
    pub shortcut: [u8; 4],
    pub adornment: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_popup_row_style(
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

impl WorkbenchPopupRowState {
    fn painter_state(self) -> UiPainterState {
        UiPainterState {
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.focused,
            disabled: self.disabled,
            checked: self.checked,
            selected: self.selected,
            ..UiPainterState::normal()
        }
    }

    fn marked(self) -> bool {
        self.checked || self.selected
    }
}

fn popup_row_background(state: UiPainterResolvedState, marked: bool, hot: bool) -> Option<[u8; 4]> {
    if state == UiPainterResolvedState::Disabled {
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
    (marked && state != UiPainterResolvedState::Disabled).then_some(PALETTE.focus_ring)
}

fn popup_row_text_color(
    row: WorkbenchPopupRowState,
    state: UiPainterResolvedState,
    marked: bool,
    hot: bool,
) -> [u8; 4] {
    if state == UiPainterResolvedState::Disabled {
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
    if state == UiPainterResolvedState::Disabled {
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
    if state == UiPainterResolvedState::Disabled {
        PALETTE.text_disabled
    } else if row.danger {
        WORKBENCH_POPUP_ROW_DANGER_TEXT
    } else if marked || hot {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}

fn is_hot(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}
