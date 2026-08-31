use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphParameterAsset, AnimationParameterValue,
};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocumentKind,
};

use super::AnimationEditorSessionError;

fn graph_bytes() -> Vec<u8> {
    AnimationGraphAsset {
        name: Some("Hero Graph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "grounded".to_string(),
            default_value: AnimationParameterValue::Bool(true),
        }],
        nodes: Vec::new(),
    }
    .to_bytes()
    .expect("graph fixture must encode")
}

#[test]
fn route_document_kind_decodes_graph_without_filename_suffix() {
    let asset =
        AnimationAuthoringAsset::from_bytes(AnimationAuthoringDocumentKind::Graph, &graph_bytes())
            .expect("the route document kind must select the graph decoder");

    assert!(matches!(asset, AnimationAuthoringAsset::Graph(_)));
}

#[test]
fn route_document_kind_reports_binary_kind_mismatch_without_error_text_matching() {
    let error = AnimationAuthoringAsset::from_bytes(
        AnimationAuthoringDocumentKind::Sequence,
        &graph_bytes(),
    )
    .map_err(AnimationEditorSessionError::from_animation_asset_error)
    .expect_err("a sequence route must reject graph binary bytes");

    assert_eq!(
        error.binary_kind_mismatch().map(|diagnostic| (
            diagnostic.expected(),
            diagnostic.actual(),
            diagnostic.code(),
        )),
        Some(("sequence", "graph", "ZR-ANIM-LOAD-001"))
    );
}
