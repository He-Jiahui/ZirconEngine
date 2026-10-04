//! typed 按钮颜色只应用于 Button/IconButton；普通节点仍由自身变体与语义决定颜色。
//! 容器、内容与轮廓读取同一主题协议，primary 判断同时兼容已有模板变体。

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
#[path = "tests/buttons.rs"]
mod tests;
