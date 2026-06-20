use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{
    chip_color_token, MUI_CHIP_DEFAULT_FILLED, MUI_ERROR_MAIN, MUI_INFO_MAIN, MUI_PRIMARY_MAIN,
    MUI_SECONDARY_MAIN, MUI_SUCCESS_MAIN, MUI_WARNING_MAIN,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_background_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            None
        } else {
            Some(match chip_color_token(node) {
                "primary" => MUI_PRIMARY_MAIN,
                "secondary" => MUI_SECONDARY_MAIN,
                "error" => MUI_ERROR_MAIN,
                "info" => MUI_INFO_MAIN,
                "success" => MUI_SUCCESS_MAIN,
                "warning" => MUI_WARNING_MAIN,
                _ => MUI_CHIP_DEFAULT_FILLED,
            })
        }
    })
}
