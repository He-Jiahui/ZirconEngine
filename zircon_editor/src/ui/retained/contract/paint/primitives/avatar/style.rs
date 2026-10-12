use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::{component_variant_contains, first_non_empty, resolved_style_color};

// 根序列按有无图像选择默认底色；样式函数先处理禁用态、再用显式样式，最后由宿主调色板补默认色。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_background_color(
    node: &TemplatePaneNodeData,
    color_default: bool,
) -> [u8; 4] {
    avatar_background_color_from_host(node, color_default, current_host_palette())
}

fn avatar_background_color_from_host(
    node: &TemplatePaneNodeData,
    color_default: bool,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if color_default || component_variant_contains(node, "colorDefault") {
            palette.surface_hover
        } else {
            palette.surface_selected
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_foreground_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    avatar_foreground_color_from_host(node, current_host_palette())
}

fn avatar_foreground_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        match first_non_empty(&[node.text_tone.as_str(), node.validation_level.as_str()]) {
            "primary" | "accent" => palette.accent,
            "muted" | "secondary" => palette.text_muted,
            "warning" => palette.warning,
            "error" | "danger" => palette.error,
            "success" => palette.success,
            "info" => palette.info,
            _ => palette.text,
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    avatar_border_color_from_host(node, current_host_palette())
}

fn avatar_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        (node.border_width > 0.0 || node.button_style.element.border_width > 0.0)
            .then_some(palette.border)
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(1.0)
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
