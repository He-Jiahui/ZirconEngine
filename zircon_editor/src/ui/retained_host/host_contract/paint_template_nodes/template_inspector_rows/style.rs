use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const RESOURCE_FIELD_BACKGROUND: [u8; 4] = [22, 28, 32, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const RESOURCE_FIELD_BORDER: [u8; 4] = [40, 50, 56, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const RESOURCE_FIELD_HOVER:
    [u8; 4] = [31, 40, 45, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_LABEL_COLOR: [u8; 4] = [174, 187, 193, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_DISCLOSURE_LABEL_COLOR: [u8; 4] = [157, 168, 174, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_VALUE_COLOR: [u8; 4] = [198, 210, 215, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_COUNT_COLOR: [u8; 4] = [153, 168, 175, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_GLYPH_COLOR: [u8; 4] = [148, 165, 173, 255];

use super::super::template_inspector_row_geometry::INSPECTOR_CHEVRON_SIZE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_value_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    declared_color(node.value_color).unwrap_or(INSPECTOR_VALUE_COLOR)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(INSPECTOR_LABEL_COLOR)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_count_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(INSPECTOR_COUNT_COLOR)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_glyph_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    declared_color(node.icon_color).unwrap_or(INSPECTOR_GLYPH_COLOR)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_chevron_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    let size = node.layout_icon_size;
    if size.is_finite() && size > 0.0 {
        size
    } else {
        INSPECTOR_CHEVRON_SIZE
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn disclosure_label_color(
) -> [u8; 4] {
    INSPECTOR_DISCLOSURE_LABEL_COLOR
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_field_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.hovered || node.pressed {
        RESOURCE_FIELD_HOVER
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .filter(|color| color[3] > 0)
            .unwrap_or(RESOURCE_FIELD_BACKGROUND)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resource_field_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .filter(|color| color[3] > 0)
        .unwrap_or(RESOURCE_FIELD_BORDER)
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
