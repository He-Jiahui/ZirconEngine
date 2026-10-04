use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_icon_node(
    node: &TemplatePaneNodeData,
) -> bool {
    !node.icon_name.is_empty() || matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_icon_only_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_has_image_source(
    node: &TemplatePaneNodeData,
) -> bool {
    if is_asset_thumbnail_visual(node) {
        return false;
    }
    node.has_preview_image || !node.media_source.is_empty() || !node.icon_name.is_empty()
}

fn is_asset_thumbnail_visual(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "asset-thumbnail-visual"
        && matches!(
            node.surface_variant.as_str(),
            "asset-placeholder-visual" | "asset-preview-visual"
        )
}

#[cfg(test)]
#[path = "identity/tests/single_match_tests.rs"]
mod single_match_tests;

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
