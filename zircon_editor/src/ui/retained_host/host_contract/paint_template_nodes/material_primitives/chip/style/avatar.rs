use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::palette::{
    chip_color_token, MUI_CHIP_DEFAULT_AVATAR, MUI_ERROR_DARK, MUI_INFO_DARK, MUI_PRIMARY_DARK,
    MUI_SECONDARY_DARK, MUI_SUCCESS_DARK, MUI_WARNING_DARK,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_avatar_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match chip_color_token(node) {
        "primary" => MUI_PRIMARY_DARK,
        "secondary" => MUI_SECONDARY_DARK,
        "error" => MUI_ERROR_DARK,
        "info" => MUI_INFO_DARK,
        "success" => MUI_SUCCESS_DARK,
        "warning" => MUI_WARNING_DARK,
        _ => MUI_CHIP_DEFAULT_AVATAR,
    }
}
