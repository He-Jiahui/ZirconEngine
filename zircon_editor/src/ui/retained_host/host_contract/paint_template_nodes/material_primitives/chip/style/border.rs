use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main_from_host};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    chip_border_color_from_host(node, current_host_palette())
}

fn chip_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            Some(
                chip_palette_main_from_host(chip_color_token(node), palette)
                    .unwrap_or(palette.border),
            )
        } else if node.border_width > 0.0 || node.button_style.element.border_width > 0.0 {
            Some(palette.border)
        } else {
            None
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(if chip_is_outlined(node) { 1.0 } else { 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn outlined_chip_border_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];
        palette.border = [20, 21, 22, 255];
        let mut node = TemplatePaneNodeData::default();
        node.component_variant = "outlined colorPrimary".into();

        assert_eq!(
            chip_border_color_from_host(&node, palette),
            Some([10, 11, 12, 255])
        );

        node.component_variant = "outlined".into();
        assert_eq!(
            chip_border_color_from_host(&node, palette),
            Some([20, 21, 22, 255])
        );
    }

    #[test]
    fn explicit_chip_border_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.border = [10, 11, 12, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(chip_border_color_from_host(&node, palette), None);

        node.border_width = 1.0;
        assert_eq!(
            chip_border_color_from_host(&node, palette),
            Some([10, 11, 12, 255])
        );
    }

    #[test]
    fn declared_chip_border_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.component_variant = "outlined colorPrimary".into();
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));

        assert_eq!(
            chip_border_color_from_host(&node, palette),
            Some([30, 31, 32, 255])
        );
    }
}
