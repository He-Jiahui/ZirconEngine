use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::{component_variant_contains, first_non_empty, resolved_style_color};
use super::tokens::badge_color_token;

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
    if component_variant_contains(node, "overlapCircular")
        || component_variant_contains(node, "circular")
    {
        background
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(background)
    }
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
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn badge_overlay_background_projects_from_host_palette() {
        let mut palette = HostMaterialPalette {
            shell_background: [1, 1, 1, 255],
            surface: [2, 2, 2, 255],
            surface_inset: [3, 3, 3, 255],
            surface_hover: [10, 11, 12, 255],
            surface_pressed: [4, 4, 4, 255],
            surface_selected: [5, 5, 5, 255],
            surface_disabled: [20, 21, 22, 255],
            accent: [30, 31, 32, 255],
            accent_soft: [40, 41, 42, 255],
            border: [6, 6, 6, 255],
            separator_strong: [7, 7, 7, 255],
            separator_soft: [8, 8, 8, 255],
            text: [200, 201, 202, 255],
            text_muted: [9, 9, 9, 255],
            text_disabled: [150, 151, 152, 255],
            warning: [50, 51, 52, 255],
            warning_container: [53, 54, 55, 255],
            error: [60, 61, 62, 255],
            error_container: [63, 64, 65, 255],
            success: [70, 71, 72, 255],
            success_container: [73, 74, 75, 255],
            info: [80, 81, 82, 255],
            info_container: [83, 84, 85, 255],
            popup: [11, 11, 11, 255],
            track: [12, 12, 12, 255],
            focus_ring: [13, 13, 13, 255],
            border_disabled: [14, 14, 14, 255],
            shadow: [15, 15, 15, 255],
        };
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.component_variant = "primary".into();
        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );

        node.component_variant = "secondary".into();
        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );

        node.component_variant = "warning".into();
        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [50, 51, 52, 255]
        );

        node.component_variant.clear();
        node.validation_level = "danger".into();
        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [60, 61, 62, 255]
        );

        palette.surface_disabled = [90, 91, 92, 255];
        node.disabled = true;
        assert_eq!(
            badge_overlay_background_color_from_host(&node, palette),
            [90, 91, 92, 255]
        );
    }

    #[test]
    fn badge_overlay_text_projects_from_host_palette() {
        let mut palette = HostMaterialPalette {
            shell_background: [1, 2, 3, 255],
            surface: [0, 0, 0, 255],
            surface_inset: [0, 0, 0, 255],
            surface_hover: [0, 0, 0, 255],
            surface_pressed: [0, 0, 0, 255],
            surface_selected: [0, 0, 0, 255],
            surface_disabled: [0, 0, 0, 255],
            accent: [0, 0, 0, 255],
            accent_soft: [0, 0, 0, 255],
            border: [0, 0, 0, 255],
            separator_strong: [0, 0, 0, 255],
            separator_soft: [0, 0, 0, 255],
            text: [4, 5, 6, 255],
            text_muted: [0, 0, 0, 255],
            text_disabled: [7, 8, 9, 255],
            warning: [0, 0, 0, 255],
            warning_container: [0, 0, 0, 255],
            error: [0, 0, 0, 255],
            error_container: [0, 0, 0, 255],
            success: [0, 0, 0, 255],
            success_container: [0, 0, 0, 255],
            info: [0, 0, 0, 255],
            info_container: [0, 0, 0, 255],
            popup: [0, 0, 0, 255],
            track: [0, 0, 0, 255],
            focus_ring: [0, 0, 0, 255],
            border_disabled: [0, 0, 0, 255],
            shadow: [0, 0, 0, 255],
        };
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            badge_overlay_text_color_from_host(&node, palette),
            [4, 5, 6, 255]
        );

        node.component_variant = "primary".into();
        assert_eq!(
            badge_overlay_text_color_from_host(&node, palette),
            [1, 2, 3, 255]
        );

        node.component_variant = "secondary".into();
        assert_eq!(
            badge_overlay_text_color_from_host(&node, palette),
            [4, 5, 6, 255]
        );

        palette.text_disabled = [10, 11, 12, 255];
        node.disabled = true;
        assert_eq!(
            badge_overlay_text_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
    }

    #[test]
    fn badge_overlay_border_preserves_declared_override() {
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(101, 102, 103, 255)));

        assert_eq!(
            badge_overlay_border_color(&node, [1, 2, 3, 255]),
            [101, 102, 103, 255]
        );

        node.component_variant = "overlapCircular".into();
        assert_eq!(
            badge_overlay_border_color(&node, [1, 2, 3, 255]),
            [1, 2, 3, 255]
        );
    }
}
