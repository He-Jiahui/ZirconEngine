use super::super::super::super::data::TemplatePaneOptionData;
use super::super::super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};

pub(super) fn command_row_style(option: &TemplatePaneOptionData) -> WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: option.hovered,
        pressed: option.pressed,
        focused: option.focused,
        disabled: option.disabled,
        selected: option.selected || option.special,
        loading: option.loading,
        ..WorkbenchPopupRowState::default()
    })
}
