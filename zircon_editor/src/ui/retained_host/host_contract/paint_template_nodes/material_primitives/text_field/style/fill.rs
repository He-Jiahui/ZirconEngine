use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::super::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_FILLED_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    field_fill_color_from_host(node, current_host_palette())
}

fn field_fill_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if node.hovered {
            palette.surface
        } else {
            palette.surface_inset
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn filled_text_field_background_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_inset = [10, 11, 12, 255];
        palette.surface = [20, 21, 22, 255];
        palette.surface_disabled = [30, 31, 32, 255];

        let mut node = TemplatePaneNodeData::default();
        assert_eq!(
            field_fill_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.hovered = true;
        assert_eq!(
            field_fill_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );

        node.disabled = true;
        assert_eq!(
            field_fill_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );
    }

    #[test]
    fn filled_text_field_declared_background_overrides_palette_when_available() {
        let mut palette = PALETTE;
        palette.surface_disabled = [30, 31, 32, 255];
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

        assert_eq!(
            field_fill_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );

        node.disabled = true;
        assert_eq!(
            field_fill_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );
    }
}
