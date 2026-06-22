use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::{component_variant_contains, resolved_style_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_STANDARD_UNDERLINE: f32 = 1.0;
const MUI_FIELD_ACTIVE_UNDERLINE: f32 = 2.0;

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
