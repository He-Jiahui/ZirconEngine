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
#[path = "tests/lookup.rs"]
mod tests;
