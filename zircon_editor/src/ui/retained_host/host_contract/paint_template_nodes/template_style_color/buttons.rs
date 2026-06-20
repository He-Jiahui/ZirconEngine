use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::roles::material_role_color;
use zircon_runtime_interface::ui::style::{ButtonColor, ButtonVariant};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn typed_button_variant_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match node.button_style.variant.normalized() {
        ButtonVariant::Contained => Some(button_container_color(&node.button_style.color)),
        ButtonVariant::Outlined => Some(PALETTE.surface_inset),
        ButtonVariant::Text | ButtonVariant::Default => None,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_primary_contained_button(
    node: &TemplatePaneNodeData,
) -> bool {
    (node.button_style.variant.normalized() == ButtonVariant::Contained
        && is_primary_button_color(&node.button_style.color))
        || matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn typed_button_tone_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
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
