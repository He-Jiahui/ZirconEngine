use super::tokens::{
    MUI_ERROR_MAIN, MUI_INFO_MAIN, MUI_ON_DARK, MUI_ON_WARNING, MUI_SUCCESS_MAIN, MUI_WARNING_MAIN,
};
use super::variants::alert_color_token;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_filled_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if alert_color_token(node) == "warning" {
        MUI_ON_WARNING
    } else {
        MUI_ON_DARK
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_main_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => MUI_SUCCESS_MAIN,
        "info" => MUI_INFO_MAIN,
        "error" | "danger" => MUI_ERROR_MAIN,
        "warning" => MUI_WARNING_MAIN,
        _ => PALETTE.info,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_container_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => PALETTE.success_container,
        "info" => PALETTE.info_container,
        "error" | "danger" => PALETTE.error_container,
        "warning" => PALETTE.warning_container,
        _ => PALETTE.info_container,
    }
}
