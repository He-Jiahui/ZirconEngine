use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chat_surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chat_surface_color_from_host(node, current_host_palette())
}

fn chat_surface_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if matches!(node.validation_level.as_str(), "error" | "danger") {
        palette.error_container
    } else if node.component_variant.as_str().contains("streaming") {
        palette.info_container
    } else {
        palette.surface_inset
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chat_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = current_host_palette();
    if node.disabled {
        palette.text_disabled
    } else {
        resolved_style_color(node.button_style.element.foreground_color.as_ref())
            .unwrap_or(palette.text)
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
