use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonVariant;

pub(super) fn variant_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match node.surface_variant.as_str() {
        "tooltip" | "snackbar" => return PALETTE.popup,
        "paper" | "paper-outlined" | "dialog" | "popover" => return PALETTE.popup,
        _ => {}
    }
    if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
    {
        return PALETTE.accent;
    }
    match node.surface_variant.as_str() {
        "inset"
        | "scroll-body"
        | "asset-tree-row"
        | "reference-row"
        | "asset-table"
        | "asset-content"
        | "content-panel"
        | "asset-placeholder"
        | "asset-placeholder-visual" => PALETTE.surface_inset,
        "asset-thumbnail-card" => [0, 0, 0, 0],
        "asset-thumbnail-name-area" => PALETTE.surface,
        "asset-type-badge" => PALETTE.surface_hover,
        "popup" | "elevated" => PALETTE.popup,
        "panel" | "component-panel" | "asset-preview" | "asset-preview-visual" => PALETTE.surface,
        "shell" => PALETTE.shell_background,
        _ => match node.role.as_str() {
            "Button" if node.surface_variant.is_empty() && is_explicit_text_button(node) => {
                [0, 0, 0, 0]
            }
            "Button" if node.surface_variant.is_empty() => PALETTE.surface_hover,
            _ => PALETTE.surface,
        },
    }
}

fn is_explicit_text_button(node: &TemplatePaneNodeData) -> bool {
    matches!(node.button_variant.as_str(), "default" | "text")
        || (!node.button_variant.is_empty()
            && node.button_style.variant.normalized() == ButtonVariant::Text)
}
