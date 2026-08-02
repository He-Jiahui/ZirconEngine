use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chat_surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chat_surface_color_from_host(node, current_host_palette())
}

fn chat_surface_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if matches!(node.validation_level.as_str(), "error" | "danger") {
        palette.error_container
    } else if node.component_variant.as_str().contains("streaming") {
        palette.info_container
    } else {
        palette.surface_inset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_chat_surface_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.error_container = [10, 11, 12, 255];
        palette.info_container = [20, 21, 22, 255];
        palette.surface_inset = [30, 31, 32, 255];

        let mut error_node = TemplatePaneNodeData::default();
        error_node.validation_level = "error".into();
        assert_eq!(
            chat_surface_color_from_host(&error_node, palette),
            [10, 11, 12, 255]
        );

        let mut streaming_node = TemplatePaneNodeData::default();
        streaming_node.component_variant = "streaming".into();
        assert_eq!(
            chat_surface_color_from_host(&streaming_node, palette),
            [20, 21, 22, 255]
        );

        assert_eq!(
            chat_surface_color_from_host(&TemplatePaneNodeData::default(), palette),
            [30, 31, 32, 255]
        );
    }
}
