use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_asset_preview_surface(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.surface_variant.as_str(),
        "asset-preview" | "asset-preview-visual" | "asset-thumbnail-name-area"
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_content_panel_surface(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.surface_variant.as_str(),
        "asset-content" | "content-panel" | "asset-thumbnail-card"
    )
}
