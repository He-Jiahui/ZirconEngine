use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::template_inspector_row_geometry::inspector_row_metrics;
use super::super::template_style_color::resolved_style_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct InspectorRowPalette {
    pub field_surface: [u8; 4],
    pub field_border: [u8; 4],
    pub field_hover: [u8; 4],
    pub label: [u8; 4],
    pub value: [u8; 4],
    pub count: [u8; 4],
    pub glyph: [u8; 4],
    pub focus_border: [u8; 4],
    pub checked_surface: [u8; 4],
    pub checked_border: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn inspector_row_palette(
) -> InspectorRowPalette {
    inspector_row_palette_from_host(current_host_palette())
}

fn inspector_row_palette_from_host(palette: HostMaterialPalette) -> InspectorRowPalette {
    InspectorRowPalette {
        field_surface: palette.surface_inset,
        field_border: palette.border,
        field_hover: palette.surface_hover,
        label: palette.text_muted,
        value: palette.text,
        count: palette.text_muted,
        glyph: palette.text_muted,
        focus_border: palette.focus_ring,
        checked_surface: palette.accent_soft,
        checked_border: palette.accent,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_value_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resource_value_color_from_palette(node, inspector_row_palette())
}

pub(super) fn resource_value_color_from_palette(
    node: &TemplatePaneNodeData,
    palette: InspectorRowPalette,
) -> [u8; 4] {
    declared_color(node.value_color).unwrap_or(palette.value)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resource_label_color_from_palette(node, inspector_row_palette())
}

pub(super) fn resource_label_color_from_palette(
    node: &TemplatePaneNodeData,
    palette: InspectorRowPalette,
) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(palette.label)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_count_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(inspector_row_palette().count)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_glyph_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resource_glyph_color_from_palette(node, inspector_row_palette())
}

pub(super) fn resource_glyph_color_from_palette(
    node: &TemplatePaneNodeData,
    palette: InspectorRowPalette,
) -> [u8; 4] {
    declared_color(node.icon_color).unwrap_or(palette.glyph)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_chevron_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    resource_chevron_size_with_default(node, inspector_row_metrics().chevron_size)
}

fn resource_chevron_size_with_default(node: &TemplatePaneNodeData, default_size: f32) -> f32 {
    let size = node.layout_icon_size;
    if size.is_finite() && size > 0.0 {
        size
    } else {
        default_size
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_field_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resource_field_background_from_palette(node, inspector_row_palette())
}

pub(super) fn resource_field_background_from_palette(
    node: &TemplatePaneNodeData,
    palette: InspectorRowPalette,
) -> [u8; 4] {
    if node.hovered || node.pressed {
        palette.field_hover
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .filter(|color| color[3] > 0)
            .unwrap_or(palette.field_surface)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_field_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resource_field_border_from_palette(node, inspector_row_palette())
}

pub(super) fn resource_field_border_from_palette(
    node: &TemplatePaneNodeData,
    palette: InspectorRowPalette,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .filter(|color| color[3] > 0)
        .unwrap_or(palette.field_border)
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

#[cfg(test)]
mod tests {
    use super::super::super::super::paint_theme::PALETTE;
    use super::*;

    #[test]
    fn inspector_row_palette_projects_from_host_material_roles() {
        let mut host = PALETTE;
        host.surface_inset = [1, 2, 3, 4];
        host.border = [5, 6, 7, 8];
        host.surface_hover = [9, 10, 11, 12];
        host.text_muted = [13, 14, 15, 16];
        host.text = [17, 18, 19, 20];
        host.focus_ring = [21, 22, 23, 24];
        host.accent_soft = [25, 26, 27, 28];
        host.accent = [29, 30, 31, 32];

        let inspector = inspector_row_palette_from_host(host);

        assert_eq!(inspector.field_surface, [1, 2, 3, 4]);
        assert_eq!(inspector.field_border, [5, 6, 7, 8]);
        assert_eq!(inspector.field_hover, [9, 10, 11, 12]);
        assert_eq!(inspector.label, [13, 14, 15, 16]);
        assert_eq!(inspector.value, [17, 18, 19, 20]);
        assert_eq!(inspector.count, [13, 14, 15, 16]);
        assert_eq!(inspector.glyph, [13, 14, 15, 16]);
        assert_eq!(inspector.focus_border, [21, 22, 23, 24]);
        assert_eq!(inspector.checked_surface, [25, 26, 27, 28]);
        assert_eq!(inspector.checked_border, [29, 30, 31, 32]);
    }

    #[test]
    fn inspector_row_defaults_follow_the_projected_host_palette_and_metric() {
        let mut host = PALETTE;
        host.text_muted = [61, 62, 63, 255];
        let palette = inspector_row_palette_from_host(host);
        let node = TemplatePaneNodeData::default();

        assert_eq!(
            resource_label_color_from_palette(&node, palette),
            [61, 62, 63, 255]
        );
        assert_eq!(
            resource_glyph_color_from_palette(&node, palette),
            [61, 62, 63, 255]
        );
        assert_eq!(resource_chevron_size_with_default(&node, 14.0), 14.0);
    }
}
