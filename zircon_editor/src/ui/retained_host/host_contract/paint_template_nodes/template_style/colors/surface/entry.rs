use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::template_style_color::{
    resolved_style_color, typed_button_variant_background,
};
use super::super::super::state::is_button_disabled;
use super::interaction::interaction_surface_color;
use super::severity::severity_surface_color;
use super::variants::variant_surface_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.surface_disabled;
    }
    if let Some(color) = severity_surface_color(node) {
        return color;
    }
    if let Some(color) = interaction_surface_color(node) {
        return color;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.background_color.as_ref()) {
        return color;
    }
    if let Some(color) = typed_button_variant_background(node) {
        return color;
    }
    variant_surface_color(node)
}
