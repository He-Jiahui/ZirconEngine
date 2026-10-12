//! 候选行状态适配共享 popup 样式；recent/special 也获得选中背景，但不改变真正的选择或执行目标。

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
