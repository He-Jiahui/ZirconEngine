//! 将菜单项状态映射为 Starship 菜单样式；悬停及按下使用主题的选中表面。

use super::super::super::super::data::TemplatePaneMenuItemData;
use super::super::super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
use super::super::super::template_popup_row_adornments::menu_item_loading_and_danger;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_menu_row_style(
    item: &TemplatePaneMenuItemData,
) -> WorkbenchPopupRowStyle {
    let (loading, danger) = menu_item_loading_and_danger(item);
    let mut style = select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: item.hovered,
        pressed: item.pressed,
        focused: item.focused,
        disabled: item.disabled,
        checked: item.checked,
        loading,
        danger,
        ..WorkbenchPopupRowState::default()
    });
    // Starship Menu.Button uses Primary for both hover and press, including
    // checked and keyboard-focused rows. Keep unavailable and danger priority.
    if (item.hovered || item.pressed)
        && !matches!(
            style.state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    {
        let palette = current_host_palette();
        style.background = Some(palette.surface_selected);
        if !danger {
            style.text = palette.text_selected;
            style.adornment = palette.text_selected;
        }
    }
    style
}
