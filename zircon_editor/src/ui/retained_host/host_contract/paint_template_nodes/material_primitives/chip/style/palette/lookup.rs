use crate::ui::retained_host::host_contract::paint_theme::HostMaterialPalette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_palette_main_from_host(
    color: &str,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    match color {
        "primary" => Some(palette.accent),
        "secondary" => Some(palette.accent_soft),
        "error" => Some(palette.error),
        "info" => Some(palette.info),
        "success" => Some(palette.success),
        "warning" => Some(palette.warning),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn chip_palette_main_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];
        palette.accent_soft = [20, 21, 22, 255];
        palette.error = [30, 31, 32, 255];
        palette.info = [40, 41, 42, 255];
        palette.success = [50, 51, 52, 255];
        palette.warning = [60, 61, 62, 255];

        assert_eq!(
            chip_palette_main_from_host("primary", palette),
            Some([10, 11, 12, 255])
        );
        assert_eq!(
            chip_palette_main_from_host("secondary", palette),
            Some([20, 21, 22, 255])
        );
        assert_eq!(
            chip_palette_main_from_host("error", palette),
            Some([30, 31, 32, 255])
        );
        assert_eq!(
            chip_palette_main_from_host("info", palette),
            Some([40, 41, 42, 255])
        );
        assert_eq!(
            chip_palette_main_from_host("success", palette),
            Some([50, 51, 52, 255])
        );
        assert_eq!(
            chip_palette_main_from_host("warning", palette),
            Some([60, 61, 62, 255])
        );
        assert_eq!(chip_palette_main_from_host("default", palette), None);
    }
}
