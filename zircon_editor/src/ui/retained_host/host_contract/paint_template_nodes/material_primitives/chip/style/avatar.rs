use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::palette::{chip_color_token, chip_palette_main_from_host};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_avatar_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chip_avatar_background_color_from_host(node, current_host_palette())
}

fn chip_avatar_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    chip_palette_main_from_host(chip_color_token(node), palette).unwrap_or(palette.surface_selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn chip_avatar_background_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_selected = [10, 11, 12, 255];
        palette.accent = [20, 21, 22, 255];
        palette.accent_soft = [30, 31, 32, 255];
        palette.warning = [40, 41, 42, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            chip_avatar_background_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.component_variant = "primary".into();
        assert_eq!(
            chip_avatar_background_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );

        node.component_variant = "secondary".into();
        assert_eq!(
            chip_avatar_background_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );

        node.component_variant = "warning".into();
        assert_eq!(
            chip_avatar_background_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );
    }
}
