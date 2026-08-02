use super::variants::alert_color_token;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn alert_main_and_container_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.success = [10, 11, 12, 255];
        palette.info = [20, 21, 22, 255];
        palette.warning = [30, 31, 32, 255];
        palette.error = [40, 41, 42, 255];
        palette.success_container = [50, 51, 52, 255];
        palette.warning_container = [60, 61, 62, 255];
        let mut node = TemplatePaneNodeData::default();

        node.component_variant = "success".into();
        assert_eq!(
            alert_main_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
        assert_eq!(
            alert_container_color_from_host(&node, palette),
            [50, 51, 52, 255]
        );

        node.component_variant = "warning".into();
        assert_eq!(
            alert_main_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );
        assert_eq!(
            alert_container_color_from_host(&node, palette),
            [60, 61, 62, 255]
        );

        node.component_variant.clear();
        node.validation_level = "danger".into();
        assert_eq!(
            alert_main_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );
    }

    #[test]
    fn alert_filled_text_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.shell_background = [70, 71, 72, 255];
        let mut node = TemplatePaneNodeData::default();
        node.component_variant = "filled warning".into();

        assert_eq!(
            alert_filled_text_color_from_host(&node, palette),
            [70, 71, 72, 255]
        );
    }
}
