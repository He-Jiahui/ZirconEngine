use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::{first_non_empty, resolved_style_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    divider_color_from_host(node, current_host_palette())
}

fn divider_color_from_host(node: &TemplatePaneNodeData, palette: HostMaterialPalette) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
        return palette.border_disabled;
    }
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .unwrap_or(palette.border)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    divider_text_color_from_host(node, current_host_palette())
}

fn divider_text_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn divider_line_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.border = [10, 11, 12, 255];
        palette.border_disabled = [20, 21, 22, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(divider_color_from_host(&node, palette), [10, 11, 12, 255]);

        node.disabled = true;
        assert_eq!(divider_color_from_host(&node, palette), [20, 21, 22, 255]);
    }

    #[test]
    fn divider_declared_line_color_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));

        assert_eq!(divider_color_from_host(&node, palette), [30, 31, 32, 255]);
    }

    #[test]
    fn divider_text_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.text = [10, 11, 12, 255];
        palette.text_disabled = [20, 21, 22, 255];
        palette.accent = [30, 31, 32, 255];
        palette.text_muted = [40, 41, 42, 255];
        palette.warning = [50, 51, 52, 255];
        palette.error = [60, 61, 62, 255];
        palette.success = [70, 71, 72, 255];
        palette.info = [80, 81, 82, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.text_tone = "primary".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );

        node.text_tone = "secondary".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );

        node.text_tone.clear();
        node.validation_level = "warning".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [50, 51, 52, 255]
        );

        node.validation_level = "error".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [60, 61, 62, 255]
        );

        node.validation_level = "success".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [70, 71, 72, 255]
        );

        node.validation_level = "info".into();
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [80, 81, 82, 255]
        );

        node.disabled = true;
        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );
    }

    #[test]
    fn divider_declared_text_color_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.text_tone = "primary".into();
        node.button_style.element.foreground_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(90, 91, 92, 255)));

        assert_eq!(
            divider_text_color_from_host(&node, palette),
            [90, 91, 92, 255]
        );
    }
}
