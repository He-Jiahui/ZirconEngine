use super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use zircon_runtime_interface::ui::style::UiStyleColor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_style_color(
    color: Option<&UiStyleColor>,
) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => material_role_color(role),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn material_role_color(
    role: &str,
) -> Option<[u8; 4]> {
    material_role_color_from_host(role, current_host_palette())
}

pub(super) fn material_role_color_from_host(
    role: &str,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    match role {
        "primary" | "accent" | "material.primary" | "material_color_primary" => {
            Some(palette.accent)
        }
        "on_primary" | "material.on_primary" | "material_color_on_primary" => {
            Some(palette.shell_background)
        }
        "surface" | "material.surface" => Some(palette.surface),
        "surface_inset" | "material.surface_inset" => Some(palette.surface_inset),
        "surface_hover" | "material.surface_hover" => Some(palette.surface_hover),
        "surface_pressed" | "material.surface_pressed" => Some(palette.surface_pressed),
        "surface_selected" | "material.surface_selected" => Some(palette.surface_selected),
        "disabled" | "material.disabled" => Some(palette.surface_disabled),
        "border" | "outline" | "material.outline" => Some(palette.border),
        "focus" | "focus_ring" | "material.focus_ring" => Some(palette.focus_ring),
        "text" | "on_surface" | "material.text" | "material.on_surface" => Some(palette.text),
        "text_muted" | "muted" | "material.text_muted" => Some(palette.text_muted),
        "text_disabled" | "material.text_disabled" => Some(palette.text_disabled),
        "warning" | "material.warning" => Some(palette.warning),
        "error" | "danger" | "material.error" => Some(palette.error),
        "success" | "material.success" => Some(palette.success),
        "info" | "material.info" => Some(palette.info),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn material_role_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];
        palette.shell_background = [20, 21, 22, 255];
        palette.surface_hover = [30, 31, 32, 255];
        palette.text_muted = [40, 41, 42, 255];
        palette.warning = [50, 51, 52, 255];

        assert_eq!(
            material_role_color_from_host("material.primary", palette),
            Some([10, 11, 12, 255])
        );
        assert_eq!(
            material_role_color_from_host("material.on_primary", palette),
            Some([20, 21, 22, 255])
        );
        assert_eq!(
            material_role_color_from_host("surface_hover", palette),
            Some([30, 31, 32, 255])
        );
        assert_eq!(
            material_role_color_from_host("muted", palette),
            Some([40, 41, 42, 255])
        );
        assert_eq!(
            material_role_color_from_host("warning", palette),
            Some([50, 51, 52, 255])
        );
        assert_eq!(material_role_color_from_host("unknown", palette), None);
    }

    #[test]
    fn resolved_style_color_keeps_declared_and_inherit_semantics() {
        assert_eq!(
            resolved_style_color(Some(&UiStyleColor::Rgba(UiRgbaColor::from_u8(
                70, 71, 72, 255,
            )))),
            Some([70, 71, 72, 255])
        );
        assert_eq!(
            resolved_style_color(Some(&UiStyleColor::Transparent)),
            Some([0, 0, 0, 0])
        );
        assert_eq!(resolved_style_color(Some(&UiStyleColor::Inherit)), None);
        assert_eq!(resolved_style_color(None), None);
    }
}
