use super::super::super::super::data::TemplatePaneOptionData;
use super::super::super::style_selector::{
    WorkbenchPopupRowState, WorkbenchPopupRowStyle, select_workbench_popup_row_style,
};

pub(super) fn popup_option_row_marked(option: &TemplatePaneOptionData) -> bool {
    option.selected || option.special
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_option_row_style(
    option: &TemplatePaneOptionData,
) -> WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: option.hovered,
        pressed: option.pressed,
        focused: option.focused,
        disabled: option.disabled,
        loading: option.loading,
        selected: popup_option_row_marked(option),
        ..WorkbenchPopupRowState::default()
    })
}
