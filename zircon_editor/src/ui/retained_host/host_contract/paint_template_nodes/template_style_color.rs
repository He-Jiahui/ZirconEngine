use super::super::data::TemplatePaneNodeData;
use super::super::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{ButtonColor, ButtonVariant, UiStyleColor};

pub(super) const MUI_TOOLTIP_BG: [u8; 4] = [97, 97, 97, 255];
pub(super) const MUI_SNACKBAR_BG: [u8; 4] = [50, 50, 50, 255];
pub(super) const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];

pub(super) fn resolved_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => material_role_color(role),
    }
}

pub(super) fn typed_button_variant_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match node.button_style.variant.normalized() {
        ButtonVariant::Contained => Some(button_container_color(&node.button_style.color)),
        ButtonVariant::Outlined => Some(PALETTE.surface_inset),
        ButtonVariant::Text | ButtonVariant::Default => None,
    }
}

pub(super) fn is_primary_contained_button(node: &TemplatePaneNodeData) -> bool {
    (node.button_style.variant.normalized() == ButtonVariant::Contained
        && is_primary_button_color(&node.button_style.color))
        || matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
}

pub(super) fn typed_button_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match &node.button_style.color {
        ButtonColor::Warning => Some(PALETTE.warning),
        ButtonColor::Error => Some(PALETTE.error),
        ButtonColor::Success => Some(PALETTE.success),
        ButtonColor::Info => Some(PALETTE.info),
        ButtonColor::Custom(color) => Some(color.to_u8()),
        ButtonColor::Style(role) => material_role_color(role),
        ButtonColor::Default | ButtonColor::Primary
            if matches!(
                node.button_style.variant.normalized(),
                ButtonVariant::Contained | ButtonVariant::Outlined
            ) =>
        {
            Some(PALETTE.focus_ring)
        }
        ButtonColor::Secondary
        | ButtonColor::Inherit
        | ButtonColor::Default
        | ButtonColor::Primary => None,
    }
}

fn button_container_color(color: &ButtonColor) -> [u8; 4] {
    match color {
        ButtonColor::Warning => PALETTE.warning_container,
        ButtonColor::Error => PALETTE.error_container,
        ButtonColor::Success => PALETTE.success_container,
        ButtonColor::Info => PALETTE.info_container,
        ButtonColor::Custom(color) => color.to_u8(),
        ButtonColor::Style(role) => material_role_color(role).unwrap_or(PALETTE.surface_selected),
        ButtonColor::Default | ButtonColor::Primary => PALETTE.accent,
        ButtonColor::Secondary | ButtonColor::Inherit => PALETTE.surface_selected,
    }
}

fn is_primary_button_color(color: &ButtonColor) -> bool {
    matches!(color, ButtonColor::Default | ButtonColor::Primary)
}

fn material_role_color(role: &str) -> Option<[u8; 4]> {
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
