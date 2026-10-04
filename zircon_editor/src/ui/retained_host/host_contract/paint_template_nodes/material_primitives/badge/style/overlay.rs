use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::{first_non_empty, resolved_style_color};
use super::tokens::badge_color_token;

// 覆盖层按语义色、禁用态和校验状态选色；文字与边框须匹配背景及圆形外观。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    badge_overlay_background_color_from_host(node, current_host_palette())
}

fn badge_overlay_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.surface_disabled;
    }
    match badge_color_token(node) {
        "primary" => palette.accent,
        "secondary" => palette.accent_soft,
        "info" => palette.info,
        "success" => palette.success,
        "warning" => palette.warning,
        "default" => palette.surface_hover,
        "error" | "danger" => palette.error,
        _ => {
            if matches!(
                first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]),
                "error" | "danger"
            ) {
                palette.error
            } else {
                palette.surface_hover
            }
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    badge_overlay_text_color_from_host(node, current_host_palette())
}

fn badge_overlay_text_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.text_disabled;
    }
    match badge_color_token(node) {
        "primary" | "info" | "success" | "warning" | "error" | "danger" => palette.shell_background,
        _ => palette.text,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_border_color(
    node: &TemplatePaneNodeData,
    background: [u8; 4],
) -> [u8; 4] {
    if badge_border_is_circular(&node.component_variant) {
        background
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(background)
    }
}

fn badge_border_is_circular(component_variant: &str) -> bool {
    component_variant
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| {
            part.eq_ignore_ascii_case("overlapCircular") || part.eq_ignore_ascii_case("circular")
        })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.border_width
        .max(node.button_style.element.border_width)
        .max(0.0)
        .min(2.0)
}

#[cfg(test)]
#[path = "overlay/tests/single_scan_border_tests.rs"]
mod single_scan_border_tests;

#[cfg(test)]
#[path = "tests/overlay.rs"]
mod tests;
