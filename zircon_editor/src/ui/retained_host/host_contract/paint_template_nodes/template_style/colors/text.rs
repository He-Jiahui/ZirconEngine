use super::super::state::{button_interaction_state, is_button_disabled};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::{
    is_primary_contained_button, resolved_style_color,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        return color;
    }
    if let Some(color) = asset_thumbnail_name_area_text_color(node) {
        return color;
    }
    if is_primary_contained_button(node)
        && matches!(
            button_interaction_state(node),
            ButtonInteractionState::Normal | ButtonInteractionState::Hover
        )
    {
        return [8, 20, 22, 255];
    }
    match node.text_tone.as_str() {
        "inverse" | "on-dark" | "tooltip" | "snackbar" => PALETTE.text,
        "muted" | "subtle" => PALETTE.text_muted,
        "accent" | "primary" | "default" => PALETTE.focus_ring,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        _ => PALETTE.text,
    }
}

fn asset_thumbnail_name_area_text_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if node.component_role.as_str() != "asset-thumbnail-name-area-text" {
        return None;
    }
    if !(node.selected || node.checked) {
        return None;
    }

    Some(match node.text_tone.as_str() {
        "muted" | "subtle" => PALETTE.text_muted,
        _ => PALETTE.text,
    })
}
