mod workbench_alert;
mod workbench_button;
mod workbench_chrome;
mod workbench_dropdown;
mod workbench_icon_button;
mod workbench_list_row;
mod workbench_popup_row;
mod workbench_segmented_control;
mod workbench_selection_control;
mod workbench_slider;
mod workbench_status_control;
mod workbench_table_row;
mod workbench_text_field;
mod workbench_toast;
mod workbench_tooltip;
mod workbench_tree_row;

use super::super::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiPainterState};

pub(in crate::ui::retained_host::host_contract::painter) fn painter_state_for_node(
    node: &TemplatePaneNodeData,
) -> UiPainterState {
    let style_state = node.button_style.interaction_state;
    UiPainterState {
        hovered: node.hovered || matches!(style_state, ButtonInteractionState::Hover),
        pressed: node.pressed
            || node.enter_pressed
            || matches!(style_state, ButtonInteractionState::Pressed),
        focused: node.focused || matches!(style_state, ButtonInteractionState::Focused),
        disabled: node.disabled
            || node.button_style.disabled
            || matches!(style_state, ButtonInteractionState::Disabled),
        checked: node.checked,
        selected: node.selected,
        open: node.popup_open,
        dragging: node.dragging,
        drop_hovered: node.drop_hovered || node.active_drag_target,
        loading: node.button_style.loading
            || matches!(style_state, ButtonInteractionState::Loading),
    }
}

pub(super) use workbench_alert::{
    select_workbench_alert_style, WorkbenchAlertStyle, WorkbenchAlertTone,
};
#[cfg(test)]
pub(super) use workbench_alert::{WORKBENCH_ALERT_INFO_SURFACE, WORKBENCH_ALERT_WARNING_SURFACE};
pub(super) use workbench_button::{
    select_workbench_button_style, WorkbenchButtonKind, WorkbenchButtonStyle, ADD_COMPONENT_GLYPH,
    ADD_COMPONENT_TEXT, OUTLINED_BORDER, OUTLINED_SURFACE, OUTLINED_TEXT, PRIMARY_SURFACE,
};
pub(super) use workbench_chrome::{
    select_workbench_chrome_style, WorkbenchChromeKind, WorkbenchChromeStyle,
};
#[cfg(test)]
pub(super) use workbench_chrome::{
    WORKBENCH_CHROME_DRAWER_BG, WORKBENCH_CHROME_PANEL_BG, WORKBENCH_CHROME_SOFT_SEPARATOR,
    WORKBENCH_CHROME_STATUS_BG, WORKBENCH_CHROME_STRONG_SEPARATOR, WORKBENCH_CHROME_TOPBAR_BG,
};
pub(super) use workbench_dropdown::{
    select_workbench_dropdown_style, WorkbenchDropdownStyle, WORKBENCH_DROPDOWN_BORDER,
    WORKBENCH_DROPDOWN_FOCUS_BORDER, WORKBENCH_DROPDOWN_PLACEHOLDER, WORKBENCH_DROPDOWN_SURFACE,
};
pub(super) use workbench_icon_button::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
    WORKBENCH_ICON_PANEL_RADIUS,
};
pub(super) use workbench_list_row::{select_workbench_list_row_style, WorkbenchListRowStyle};
pub(super) use workbench_popup_row::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
    WORKBENCH_POPUP_ROW_DANGER_TEXT,
};
pub(super) use workbench_segmented_control::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind,
    WorkbenchSegmentedControlStyle, WORKBENCH_SEGMENT_IDLE_BACKGROUND,
};
pub(super) use workbench_selection_control::{
    select_workbench_selection_control_style, WorkbenchSelectionControlKind,
    WorkbenchSelectionControlStyle, WORKBENCH_CHECKBOX_CHECKED_FILL,
    WORKBENCH_RADIO_CHECKED_BORDER, WORKBENCH_RADIO_CHECKED_FILL, WORKBENCH_SELECTION_LABEL_MUTED,
    WORKBENCH_SELECTION_MARK_IDLE_BORDER, WORKBENCH_SELECTION_MARK_IDLE_FILL,
};
pub(super) use workbench_slider::{
    is_workbench_slider_state_hot, select_workbench_slider_style, WorkbenchSliderStyle,
    WORKBENCH_SLIDER_HALO, WORKBENCH_SLIDER_TEXT, WORKBENCH_SLIDER_THUMB, WORKBENCH_SLIDER_TICK,
    WORKBENCH_SLIDER_TRACK, WORKBENCH_SLIDER_TRACK_DISABLED,
};
pub(super) use workbench_status_control::{
    select_workbench_status_chip_style, select_workbench_status_icon_button_style,
    select_workbench_status_signal_style, WorkbenchStatusSignalKind, WorkbenchStatusSignalStyle,
};
#[cfg(test)]
pub(super) use workbench_status_control::{
    WORKBENCH_STATUS_NO_ERRORS_FILL, WORKBENCH_STATUS_RIGHT_BORDER,
};
pub(super) use workbench_table_row::{select_workbench_table_row_style, WorkbenchTableRowStyle};
#[cfg(test)]
pub(super) use workbench_table_row::{
    WORKBENCH_TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT, WORKBENCH_TABLE_HOVER_BG,
    WORKBENCH_TABLE_ROW_BG, WORKBENCH_TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR,
    WORKBENCH_TABLE_TAIL_BG,
};
pub(super) use workbench_text_field::{select_workbench_text_field_style, WorkbenchTextFieldStyle};
#[cfg(test)]
pub(super) use workbench_text_field::{
    WORKBENCH_TEXT_FIELD_BORDER, WORKBENCH_TEXT_FIELD_DISABLED_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE, WORKBENCH_TEXT_FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER, WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_PLACEHOLDER, WORKBENCH_TEXT_FIELD_SURFACE,
};
pub(super) use workbench_toast::select_workbench_toast_style;
#[cfg(test)]
pub(super) use workbench_toast::{
    WORKBENCH_TOAST_ACTION, WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
};
pub(super) use workbench_tooltip::select_workbench_tooltip_style;
#[cfg(test)]
pub(super) use workbench_tooltip::{
    WORKBENCH_TOOLTIP_BODY, WORKBENCH_TOOLTIP_BORDER, WORKBENCH_TOOLTIP_ICON,
    WORKBENCH_TOOLTIP_SURFACE,
};
#[cfg(test)]
pub(super) use workbench_tree_row::WORKBENCH_TREE_ROW_TEXT_SELECTED;
pub(super) use workbench_tree_row::{select_workbench_tree_row_style, WorkbenchTreeRowStyle};
