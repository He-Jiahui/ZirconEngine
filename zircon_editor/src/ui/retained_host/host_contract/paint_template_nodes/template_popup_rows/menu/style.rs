use super::super::super::super::data::TemplatePaneMenuItemData;
use super::super::super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
use super::super::super::template_popup_row_adornments::menu_item_loading_and_danger;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_menu_row_style(
    item: &TemplatePaneMenuItemData,
) -> WorkbenchPopupRowStyle {
    let (loading, danger) = menu_item_loading_and_danger(item);
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: item.hovered,
        pressed: item.pressed,
        focused: item.focused,
        disabled: item.disabled,
        checked: item.checked,
        loading,
        danger,
        ..WorkbenchPopupRowState::default()
    })
}
