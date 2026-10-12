//! 文字颜色读取当前主题，并尊重已解析的前景声明；资源名称选中态保留可读性，主按钮空闲态使用反色内容。

use super::super::state::{button_interaction_state, is_button_disabled};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::{
    is_primary_contained_button, resolved_style_color,
};
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    text_color_from_palette(node, current_host_palette())
}

fn text_color_from_palette(node: &TemplatePaneNodeData, palette: HostMaterialPalette) -> [u8; 4] {
    if is_button_disabled(node) {
        return palette.text_disabled;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        return color;
    }
    if let Some(color) = asset_thumbnail_name_area_text_color(node, palette) {
        return color;
    }
    if is_primary_contained_button(node)
        && matches!(
            button_interaction_state(node),
            ButtonInteractionState::Normal | ButtonInteractionState::Hover
        )
    {
        return palette.shell_background;
    }
    match node.text_tone.as_str() {
        "inverse" | "on-dark" | "tooltip" | "snackbar" => palette.text,
        "muted" | "subtle" => palette.text_muted,
        "accent" | "primary" | "default" => palette.accent,
        "warning" => palette.warning,
        "error" | "danger" => palette.error,
        "success" => palette.success,
        "info" => palette.info,
        _ => palette.text,
    }
}

fn asset_thumbnail_name_area_text_color(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    if node.component_role.as_str() != "asset-thumbnail-name-area-text" {
        return None;
    }
    if !(node.selected || node.checked) {
        return None;
    }

    Some(match node.text_tone.as_str() {
        "muted" | "subtle" => palette.text_muted,
        _ => palette.text,
    })
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
