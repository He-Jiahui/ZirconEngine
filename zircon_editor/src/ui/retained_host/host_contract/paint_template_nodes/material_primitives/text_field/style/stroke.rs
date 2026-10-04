use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::super::style_selector::focus_visible_for_node;
use super::super::super::{component_variant_contains, resolved_style_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_STANDARD_UNDERLINE: f32 = 1.0;
const MUI_FIELD_ACTIVE_UNDERLINE: f32 = 2.0;

/// 状态色按禁用、校验错误、显式边框、可见焦点依次优先，供描边和底线共用。
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
#[path = "tests/stroke.rs"]
mod tests;
