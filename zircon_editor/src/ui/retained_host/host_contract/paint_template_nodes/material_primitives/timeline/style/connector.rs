use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::super::resolved_style_color;
use super::tokens::timeline_neutral_color_from_host;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_connector_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    timeline_connector_color_from_host(node, current_host_palette())
}

fn timeline_connector_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .or_else(|| resolved_style_color(node.button_style.element.border_color.as_ref()))
        .unwrap_or_else(|| timeline_neutral_color_from_host(palette))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn timeline_connector_projects_default_from_host_palette() {
        let mut palette = PALETTE;
        palette.separator_strong = [10, 11, 12, 255];
        let node = TemplatePaneNodeData::default();

        assert_eq!(
            timeline_connector_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
    }

    #[test]
    fn timeline_connector_declared_color_overrides_palette() {
        let mut palette = PALETTE;
        palette.separator_strong = [10, 11, 12, 255];
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.foreground_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));

        assert_eq!(
            timeline_connector_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );
    }
}
