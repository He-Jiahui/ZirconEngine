use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::super::style_selector::focus_visible_for_node;
use super::super::super::{component_variant_contains, resolved_style_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_STANDARD_UNDERLINE: f32 = 1.0;
const MUI_FIELD_ACTIVE_UNDERLINE: f32 = 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_stroke_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    field_stroke_color_from_host(node, current_host_palette())
}

fn field_stroke_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.border_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        return palette.error;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        return color;
    }
    if focus_visible_for_node(node) || component_variant_contains(node, "focused") {
        return palette.focus_ring;
    }
    palette.border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_stroke_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if focus_visible_for_node(node)
        || component_variant_contains(node, "focused")
        || matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        configured.max(MUI_FIELD_ACTIVE_UNDERLINE)
    } else {
        configured.max(MUI_FIELD_STANDARD_UNDERLINE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn text_field_stroke_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.border = [10, 11, 12, 255];
        palette.border_disabled = [20, 21, 22, 255];
        palette.error = [30, 31, 32, 255];
        palette.focus_ring = [40, 41, 42, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            field_stroke_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.focused = true;
        assert_eq!(
            field_stroke_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );

        node.focused = false;
        node.validation_level = "error".into();
        assert_eq!(
            field_stroke_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );

        node.validation_level.clear();
        node.disabled = true;
        assert_eq!(
            field_stroke_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );
    }

    #[test]
    fn text_field_declared_stroke_overrides_palette_when_available() {
        let palette = PALETTE;
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(50, 51, 52, 255)));

        assert_eq!(
            field_stroke_color_from_host(&node, palette),
            [50, 51, 52, 255]
        );
    }
}
