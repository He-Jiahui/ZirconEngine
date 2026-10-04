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
