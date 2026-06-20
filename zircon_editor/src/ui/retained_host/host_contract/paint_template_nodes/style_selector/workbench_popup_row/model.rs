use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiPainterState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowState
{
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

impl WorkbenchPopupRowState {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn painter_state(
        self,
    ) -> UiPainterState {
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

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn marked(self) -> bool {
        self.checked || self.selected
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowStyle
{
    pub background: Option<[u8; 4]>,
    pub selection_mark: Option<[u8; 4]>,
    pub text: [u8; 4],
    pub shortcut: [u8; 4],
    pub adornment: [u8; 4],
    pub state: UiPainterResolvedState,
}
