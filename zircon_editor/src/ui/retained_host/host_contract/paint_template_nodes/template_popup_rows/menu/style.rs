use super::super::super::super::data::TemplatePaneMenuItemData;
use super::super::super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
use super::super::super::template_popup_row_adornments::menu_item_has_flag;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_menu_row_style(
    item: &TemplatePaneMenuItemData,
) -> WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: item.hovered,
        pressed: item.pressed,
        focused: item.focused,
        disabled: item.disabled,
        checked: item.checked,
        loading: item.loading || menu_item_has_flag(item, "loading"),
        danger: menu_item_has_flag(item, "danger"),
        ..WorkbenchPopupRowState::default()
    })
}
