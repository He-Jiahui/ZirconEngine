use super::super::style_selector::WorkbenchPopupRowStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct PopupRowContentStyle {
    pub text: [u8; 4],
    pub shortcut: [u8; 4],
    pub adornment: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_content_style(
    style: &WorkbenchPopupRowStyle,
) -> PopupRowContentStyle {
    PopupRowContentStyle {
        text: style.text,
        shortcut: style.shortcut,
        adornment: style.adornment,
    }
}
