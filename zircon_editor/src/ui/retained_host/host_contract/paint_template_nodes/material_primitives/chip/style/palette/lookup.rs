use super::constants::{
    MUI_ERROR_MAIN, MUI_INFO_MAIN, MUI_PRIMARY_MAIN, MUI_SECONDARY_MAIN, MUI_SUCCESS_MAIN,
    MUI_WARNING_MAIN,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_palette_main(
    color: &str,
) -> Option<[u8; 4]> {
    match color {
        "primary" => Some(MUI_PRIMARY_MAIN),
        "secondary" => Some(MUI_SECONDARY_MAIN),
        "error" => Some(MUI_ERROR_MAIN),
        "info" => Some(MUI_INFO_MAIN),
        "success" => Some(MUI_SUCCESS_MAIN),
        "warning" => Some(MUI_WARNING_MAIN),
        _ => None,
    }
}
