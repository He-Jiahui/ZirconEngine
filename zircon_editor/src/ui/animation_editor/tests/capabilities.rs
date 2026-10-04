use super::{
    animation_editor_capability_table, resolve_animation_graph_node_kind,
    AnimationEditorCommandRejectionReason,
};
use crate::core::editing::animation_document::AnimationGraphNodeKind;

#[test]
fn capability_table_exposes_current_animation_authoring_truth() {
    let table = animation_editor_capability_table();
    let available = table
        .iter()
        .filter(|descriptor| descriptor.available())
        .map(|descriptor| descriptor.id())
        .collect::<Vec<_>>();

    assert!(available.contains(&"animation.document.open"));
    assert!(available.contains(&"animation.graph.node.output"));
    assert!(available.contains(&"animation.graph.node.blend"));
    assert!(!available.contains(&"animation.graph.node.clip"));
    assert!(!available.contains(&"animation.compiler.semantic"));
    assert!(!available.contains(&"animation.preview.runtime"));
}

#[test]
fn graph_node_resolution_uses_the_capability_table_for_typed_rejections() {
    assert_eq!(
        resolve_animation_graph_node_kind("BLEND"),
        Ok(AnimationGraphNodeKind::Blend)
    );

    let unavailable = resolve_animation_graph_node_kind("clip")
        .expect_err("declared but unavailable node kinds must be rejected");
    assert_eq!(unavailable.code(), "ZR-ANIM-CMD-002");
    assert_eq!(
        unavailable.reason(),
        AnimationEditorCommandRejectionReason::UnavailableGraphNodeKind
    );

    let unknown = resolve_animation_graph_node_kind("pose_cache")
        .expect_err("unknown node kinds must be rejected");
    assert_eq!(unknown.code(), "ZR-ANIM-CMD-001");
    assert_eq!(
        unknown.reason(),
        AnimationEditorCommandRejectionReason::UnknownGraphNodeKind
    );
}
