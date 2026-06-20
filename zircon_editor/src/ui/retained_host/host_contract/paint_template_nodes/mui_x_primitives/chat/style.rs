use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chat_surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error_container
    } else if node.component_variant.as_str().contains("streaming") {
        PALETTE.info_container
    } else {
        PALETTE.surface_inset
    }
}
