//! 没有更强状态或显式样式时，模板变体选择容器语义；资源卡片透明底交给独立名称区与图像呈现。

use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::ButtonVariant;

pub(super) fn variant_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match node.surface_variant.as_str() {
        "tooltip" | "snackbar" => return current_host_palette().popup,
        "paper" | "paper-outlined" | "dialog" | "popover" => return current_host_palette().popup,
        _ => {}
    }
    if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
    {
        return current_host_palette().accent;
    }
    match node.surface_variant.as_str() {
        "transparent" => [0, 0, 0, 0],
        "inset"
        | "scroll-body"
        | "asset-tree-row"
        | "reference-row"
        | "asset-table"
        | "asset-content"
        | "content-panel"
        | "asset-placeholder"
        | "asset-placeholder-visual" => current_host_palette().surface_inset,
        "asset-thumbnail-card" => [0, 0, 0, 0],
        "asset-thumbnail-name-area" => current_host_palette().surface,
        "asset-type-badge" => current_host_palette().surface_hover,
        "popup" | "elevated" => current_host_palette().popup,
        "panel" | "component-panel" | "asset-preview" | "asset-preview-visual" => {
            current_host_palette().surface
        }
        "shell" => current_host_palette().shell_background,
        _ => match node.role.as_str() {
            "Button" if node.surface_variant.is_empty() && is_explicit_text_button(node) => {
                [0, 0, 0, 0]
            }
            "Button" if node.surface_variant.is_empty() => current_host_palette().surface_hover,
            _ => current_host_palette().surface,
        },
    }
}

fn is_explicit_text_button(node: &TemplatePaneNodeData) -> bool {
    matches!(node.button_variant.as_str(), "default" | "text")
        || (!node.button_variant.is_empty()
            && node.button_style.variant.normalized() == ButtonVariant::Text)
}
