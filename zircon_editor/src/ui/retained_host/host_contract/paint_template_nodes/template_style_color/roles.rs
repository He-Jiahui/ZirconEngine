use super::super::super::paint_theme::PALETTE;
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
    match role {
        "primary" | "accent" | "material.primary" | "material_color_primary" => {
            Some(PALETTE.accent)
        }
        "on_primary" | "material.on_primary" | "material_color_on_primary" => {
            Some([8, 20, 22, 255])
        }
        "surface" | "material.surface" => Some(PALETTE.surface),
        "surface_inset" | "material.surface_inset" => Some(PALETTE.surface_inset),
        "surface_hover" | "material.surface_hover" => Some(PALETTE.surface_hover),
        "surface_pressed" | "material.surface_pressed" => Some(PALETTE.surface_pressed),
        "surface_selected" | "material.surface_selected" => Some(PALETTE.surface_selected),
        "disabled" | "material.disabled" => Some(PALETTE.surface_disabled),
        "border" | "outline" | "material.outline" => Some(PALETTE.border),
        "focus" | "focus_ring" | "material.focus_ring" => Some(PALETTE.focus_ring),
        "text" | "on_surface" | "material.text" | "material.on_surface" => Some(PALETTE.text),
        "text_muted" | "muted" | "material.text_muted" => Some(PALETTE.text_muted),
        "text_disabled" | "material.text_disabled" => Some(PALETTE.text_disabled),
        "warning" | "material.warning" => Some(PALETTE.warning),
        "error" | "danger" | "material.error" => Some(PALETTE.error),
        "success" | "material.success" => Some(PALETTE.success),
        "info" | "material.info" => Some(PALETTE.info),
        _ => None,
    }
}
