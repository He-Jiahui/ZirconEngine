use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, resolved_style_color};

const MUI_FIELD_FILLED_BACKGROUND: [u8; 4] = [255, 255, 255, 23];
const MUI_FIELD_FILLED_HOVER_BACKGROUND: [u8; 4] = [255, 255, 255, 31];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_STANDARD_UNDERLINE: f32 = 1.0;
const MUI_FIELD_ACTIVE_UNDERLINE: f32 = 2.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_OUTLINED_RADIUS: f32 = 4.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_FILLED_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_stroke_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.border_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        return PALETTE.error;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        return color;
    }
    if node.focused || component_variant_contains(node, "focused") {
        return PALETTE.focus_ring;
    }
    PALETTE.border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_stroke_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if node.focused
        || component_variant_contains(node, "focused")
        || matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        configured.max(MUI_FIELD_ACTIVE_UNDERLINE)
    } else {
        configured.max(MUI_FIELD_STANDARD_UNDERLINE)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if node.hovered {
            MUI_FIELD_FILLED_HOVER_BACKGROUND
        } else {
            MUI_FIELD_FILLED_BACKGROUND
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn configured_radius(
    node: &TemplatePaneNodeData,
    fallback: f32,
) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured > 0.0 {
        configured
    } else {
        fallback
    }
}
