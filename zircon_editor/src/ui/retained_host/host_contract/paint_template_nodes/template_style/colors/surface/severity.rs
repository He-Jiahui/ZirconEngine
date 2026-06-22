use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;

pub(super) fn severity_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return Some(PALETTE.error_container);
    }
    if node.validation_level.as_str() == "warning" {
        return Some(PALETTE.warning_container);
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return Some(PALETTE.success_container);
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return Some(PALETTE.info_container);
    }
    None
}
