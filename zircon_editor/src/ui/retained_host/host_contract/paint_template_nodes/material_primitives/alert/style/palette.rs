use super::variants::alert_color_token;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_filled_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    alert_filled_text_color_from_host(node, current_host_palette())
}

fn alert_filled_text_color_from_host(
    _node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.shell_background
}

// severity 令牌选择宿主主色；无显式背景的 filled、无显式边色的 outlined，以及未禁用且无显式前景的非 filled 文本才用它作回退。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_main_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    alert_main_color_from_host(node, current_host_palette())
}

fn alert_main_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => palette.success,
        "info" => palette.info,
        "error" | "danger" => palette.error,
        "warning" => palette.warning,
        _ => palette.info,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_container_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    alert_container_color_from_host(node, current_host_palette())
}

fn alert_container_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => palette.success_container,
        "info" => palette.info_container,
        "error" | "danger" => palette.error_container,
        "warning" => palette.warning_container,
        _ => palette.info_container,
    }
}

#[cfg(test)]
#[path = "tests/palette.rs"]
mod tests;
