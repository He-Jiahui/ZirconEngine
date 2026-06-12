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
    pub open: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub loading: bool,
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
            open: self.open,
            dragging: self.dragging,
            drop_hovered: self.drop_hovered,
            loading: self.loading,
            ..UiPainterState::normal()
        }
    }

    fn marked(self) -> bool {
        self.checked || self.selected
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

fn is_hot(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

fn is_unavailable(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_row_selector_projects_full_semantic_state() {
        let open = select_workbench_popup_row_style(WorkbenchPopupRowState {
            open: true,
            ..WorkbenchPopupRowState::default()
        });
        assert_eq!(open.state, UiPainterResolvedState::Open);
        assert_eq!(open.background, Some(PALETTE.surface_hover));
        assert_eq!(open.text, PALETTE.focus_ring);

        let dragging = select_workbench_popup_row_style(WorkbenchPopupRowState {
            dragging: true,
            ..WorkbenchPopupRowState::default()
        });
        assert_eq!(dragging.state, UiPainterResolvedState::Dragging);
        assert_eq!(dragging.background, Some(PALETTE.surface_hover));
        assert_eq!(dragging.adornment, PALETTE.focus_ring);

        let drop_hovered = select_workbench_popup_row_style(WorkbenchPopupRowState {
            drop_hovered: true,
            ..WorkbenchPopupRowState::default()
        });
        assert_eq!(drop_hovered.state, UiPainterResolvedState::DropHovered);
        assert_eq!(drop_hovered.background, Some(PALETTE.surface_hover));
        assert_eq!(drop_hovered.shortcut, PALETTE.text_muted);
    }

    #[test]
    fn popup_row_loading_state_uses_unavailable_visuals() {
        let loading_selected = select_workbench_popup_row_style(WorkbenchPopupRowState {
            loading: true,
            hovered: true,
            selected: true,
            danger: true,
            ..WorkbenchPopupRowState::default()
        });

        assert_eq!(loading_selected.state, UiPainterResolvedState::Loading);
        assert_eq!(loading_selected.background, None);
        assert_eq!(loading_selected.selection_mark, None);
        assert_eq!(loading_selected.text, PALETTE.text_disabled);
        assert_eq!(loading_selected.shortcut, PALETTE.text_disabled);
        assert_eq!(loading_selected.adornment, PALETTE.text_disabled);
    }
}
