use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::roles::material_role_color_from_host;
use zircon_runtime_interface::ui::style::{ButtonColor, ButtonVariant};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn typed_button_variant_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    typed_button_variant_background_from_host(node, current_host_palette())
}

fn typed_button_variant_background_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match node.button_style.variant.normalized() {
        ButtonVariant::Contained => Some(button_container_color_from_host(
            &node.button_style.color,
            palette,
        )),
        ButtonVariant::Outlined => Some(palette.surface_inset),
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
    typed_button_tone_color_from_host(node, current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn typed_button_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    typed_button_border_color_from_host(node, current_host_palette())
}

fn typed_button_tone_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match &node.button_style.color {
        ButtonColor::Warning => Some(palette.warning),
        ButtonColor::Error => Some(palette.error),
        ButtonColor::Success => Some(palette.success),
        ButtonColor::Info => Some(palette.info),
        ButtonColor::Custom(color) => Some(color.to_u8()),
        ButtonColor::Style(role) => material_role_color_from_host(role, palette),
        ButtonColor::Default | ButtonColor::Primary
            if node.button_style.variant.normalized() == ButtonVariant::Contained =>
        {
            Some(palette.shell_background)
        }
        ButtonColor::Default | ButtonColor::Primary
            if node.button_style.variant.normalized() == ButtonVariant::Outlined =>
        {
            Some(palette.accent)
        }
        ButtonColor::Secondary
        | ButtonColor::Inherit
        | ButtonColor::Default
        | ButtonColor::Primary => None,
    }
}

fn typed_button_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    if !matches!(node.role.as_str(), "Button" | "IconButton") {
        return None;
    }
    match &node.button_style.color {
        ButtonColor::Warning => Some(palette.warning),
        ButtonColor::Error => Some(palette.error),
        ButtonColor::Success => Some(palette.success),
        ButtonColor::Info => Some(palette.info),
        ButtonColor::Custom(color) => Some(color.to_u8()),
        ButtonColor::Style(role) => material_role_color_from_host(role, palette),
        ButtonColor::Default | ButtonColor::Primary
            if matches!(
                node.button_style.variant.normalized(),
                ButtonVariant::Contained | ButtonVariant::Outlined
            ) =>
        {
            Some(palette.accent)
        }
        ButtonColor::Secondary
        | ButtonColor::Inherit
        | ButtonColor::Default
        | ButtonColor::Primary => None,
    }
}

fn button_container_color_from_host(color: &ButtonColor, palette: HostMaterialPalette) -> [u8; 4] {
    match color {
        ButtonColor::Warning => palette.warning_container,
        ButtonColor::Error => palette.error_container,
        ButtonColor::Success => palette.success_container,
        ButtonColor::Info => palette.info_container,
        ButtonColor::Custom(color) => color.to_u8(),
        ButtonColor::Style(role) => {
            material_role_color_from_host(role, palette).unwrap_or(palette.surface_selected)
        }
        ButtonColor::Default | ButtonColor::Primary => palette.accent,
        ButtonColor::Secondary | ButtonColor::Inherit => palette.surface_selected,
    }
}

fn is_primary_button_color(color: &ButtonColor) -> bool {
    matches!(color, ButtonColor::Default | ButtonColor::Primary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn typed_button_background_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.warning_container = [10, 11, 12, 255];
        palette.surface_inset = [20, 21, 22, 255];
        palette.text_muted = [30, 31, 32, 255];
        let mut node = TemplatePaneNodeData::default();
        node.role = "Button".into();

        node.button_style.variant = ButtonVariant::Contained;
        node.button_style.color = ButtonColor::Warning;
        assert_eq!(
            typed_button_variant_background_from_host(&node, palette),
            Some([10, 11, 12, 255])
        );

        node.button_style.variant = ButtonVariant::Outlined;
        assert_eq!(
            typed_button_variant_background_from_host(&node, palette),
            Some([20, 21, 22, 255])
        );

        node.button_style.variant = ButtonVariant::Contained;
        node.button_style.color = ButtonColor::Style("muted".into());
        assert_eq!(
            typed_button_variant_background_from_host(&node, palette),
            Some([30, 31, 32, 255])
        );

        node.role = "Text".into();
        assert_eq!(
            typed_button_variant_background_from_host(&node, palette),
            None
        );
    }

    #[test]
    fn typed_button_tone_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.warning = [10, 11, 12, 255];
        palette.text_muted = [20, 21, 22, 255];
        palette.accent = [30, 31, 32, 255];
        palette.shell_background = [40, 41, 42, 255];
        let mut node = TemplatePaneNodeData::default();
        node.role = "IconButton".into();

        node.button_style.color = ButtonColor::Warning;
        assert_eq!(
            typed_button_tone_color_from_host(&node, palette),
            Some([10, 11, 12, 255])
        );

        node.button_style.color = ButtonColor::Style("muted".into());
        assert_eq!(
            typed_button_tone_color_from_host(&node, palette),
            Some([20, 21, 22, 255])
        );

        node.button_style.variant = ButtonVariant::Contained;
        node.button_style.color = ButtonColor::Default;
        assert_eq!(
            typed_button_tone_color_from_host(&node, palette),
            Some([40, 41, 42, 255])
        );

        node.button_style.variant = ButtonVariant::Outlined;
        assert_eq!(
            typed_button_tone_color_from_host(&node, palette),
            Some([30, 31, 32, 255])
        );

        node.button_style.color = ButtonColor::Secondary;
        assert_eq!(typed_button_tone_color_from_host(&node, palette), None);
    }

    #[test]
    fn typed_primary_border_uses_accent_without_borrowing_focus_ring() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];
        palette.focus_ring = [20, 21, 22, 255];
        let mut node = TemplatePaneNodeData::default();
        node.role = "Button".into();
        node.button_style.variant = ButtonVariant::Contained;
        node.button_style.color = ButtonColor::Primary;

        assert_eq!(
            typed_button_border_color_from_host(&node, palette),
            Some(palette.accent)
        );
        assert_ne!(
            typed_button_border_color_from_host(&node, palette),
            Some(palette.focus_ring)
        );
    }
}
