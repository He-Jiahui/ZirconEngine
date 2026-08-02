use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::resolved_style_color;

const SKELETON_DISABLED_OPACITY: f32 = 0.56;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    skeleton_color_from_host(node, current_host_palette())
}

fn skeleton_color_from_host(node: &TemplatePaneNodeData, palette: HostMaterialPalette) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(palette.surface_hover)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_wave_color()
-> [u8; 4] {
    skeleton_wave_color_from_host(current_host_palette())
}

fn skeleton_wave_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.separator_soft
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    skeleton_border_color_from_host(node, current_host_palette())
}

fn skeleton_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (skeleton_border_width(node) > 0.0).then_some(palette.surface_hover))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let width = node
        .button_style
        .element
        .border_width
        .max(node.border_width);
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn skeleton_fill_border_and_wave_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_hover = [10, 11, 12, 255];
        palette.separator_soft = [20, 21, 22, 128];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(skeleton_color_from_host(&node, palette), [10, 11, 12, 255]);
        assert_eq!(skeleton_wave_color_from_host(palette), [20, 21, 22, 128]);
        assert_eq!(skeleton_border_color_from_host(&node, palette), None);

        node.border_width = 1.0;
        assert_eq!(
            skeleton_border_color_from_host(&node, palette),
            Some([10, 11, 12, 255])
        );
    }

    #[test]
    fn skeleton_declared_fill_and_border_override_palette_when_available() {
        let mut palette = PALETTE;
        palette.surface_hover = [10, 11, 12, 255];
        let mut node = TemplatePaneNodeData::default();
        node.border_width = 1.0;
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

        assert_eq!(skeleton_color_from_host(&node, palette), [30, 31, 32, 255]);
        assert_eq!(
            skeleton_border_color_from_host(&node, palette),
            Some([40, 41, 42, 255])
        );
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_opacity(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.disabled {
        SKELETON_DISABLED_OPACITY
    } else {
        1.0
    }
}
