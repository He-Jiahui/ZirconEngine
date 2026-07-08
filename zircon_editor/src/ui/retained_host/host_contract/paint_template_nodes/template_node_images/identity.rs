use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_icon_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon") || !node.icon_name.is_empty()
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
mod tests {
    use super::*;

    #[test]
    fn asset_thumbnail_visual_preview_source_is_owned_by_asset_visual_painter() {
        let node = TemplatePaneNodeData {
            component_role: "asset-thumbnail-visual".into(),
            surface_variant: "asset-preview-visual".into(),
            media_source: "docs/tests/editor/asset-preview.png".into(),
            has_preview_image: true,
            ..TemplatePaneNodeData::default()
        };

        assert!(!template_node_has_image_source(&node));
    }

    #[test]
    fn ordinary_image_nodes_keep_generic_image_painter_source() {
        let node = TemplatePaneNodeData {
            role: "Image".into(),
            media_source: "ui/editor/showcase_checker.svg".into(),
            has_preview_image: true,
            ..TemplatePaneNodeData::default()
        };

        assert!(template_node_has_image_source(&node));
    }
}
