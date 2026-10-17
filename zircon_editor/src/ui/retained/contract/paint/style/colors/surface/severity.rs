//! 校验信息先于普通交互和显式背景，保证错误/警告的容器语义仍可见。

use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::current_host_palette;

pub(super) fn severity_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return Some(current_host_palette().error_container);
    }
    if node.validation_level.as_str() == "warning" {
        return Some(current_host_palette().warning_container);
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return Some(current_host_palette().success_container);
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return Some(current_host_palette().info_container);
    }
    None
}
