use crate::ui::{
    dispatch::UiDispatchPhase,
    event_ui::{UiNodeId, UiNodePath},
    layout::UiPoint,
    surface::{UiHitPath, UiHitTestQuery},
    tree::{UiCursor, UiPointerEvents, UiTreeNode},
};

#[test]
fn pointer_events_keep_self_child_and_passthrough_semantics_distinct() {
    assert!(UiPointerEvents::Auto.allows_self_hit_test());
    assert!(UiPointerEvents::Auto.allows_child_hit_test());
    assert!(!UiPointerEvents::Auto.is_passthrough());

    assert!(!UiPointerEvents::None.allows_self_hit_test());
    assert!(!UiPointerEvents::None.allows_child_hit_test());
    assert!(!UiPointerEvents::None.is_passthrough());

    assert!(!UiPointerEvents::SelfNone.allows_self_hit_test());
    assert!(UiPointerEvents::SelfNone.allows_child_hit_test());
    assert!(!UiPointerEvents::SelfNone.is_passthrough());

    assert!(UiPointerEvents::Pass.allows_self_hit_test());
    assert!(UiPointerEvents::Pass.allows_child_hit_test());
    assert!(UiPointerEvents::Pass.is_passthrough());

    assert_eq!(
        serde_json::to_value(UiPointerEvents::SelfNone).expect("pointer events serialize"),
        "self-none"
    );
    assert_eq!(
        serde_json::from_value::<UiPointerEvents>(serde_json::json!("self-none"))
            .expect("kebab-case pointer events deserialize"),
        UiPointerEvents::SelfNone
    );
    assert_eq!(
        serde_json::to_value(UiCursor::ResizeEw).expect("cursor serialize"),
        "resize-ew"
    );
    assert_eq!(
        serde_json::from_value::<UiCursor>(serde_json::json!("resize-ew"))
            .expect("kebab-case cursor deserializes"),
        UiCursor::ResizeEw
    );
}

#[test]
fn tree_nodes_preserve_authored_pointer_events_and_cursor() {
    let default_node = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"));
    let node = UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/child"))
        .with_pointer_events(UiPointerEvents::SelfNone)
        .with_cursor(UiCursor::ResizeEw);

    assert_eq!(default_node.pointer_events, UiPointerEvents::Auto);
    assert_eq!(default_node.cursor, None);
    assert!(default_node.allows_self_pointer_hit_test());
    assert!(default_node.allows_child_pointer_hit_test());
    assert_eq!(node.pointer_events, UiPointerEvents::SelfNone);
    assert_eq!(node.cursor, Some(UiCursor::ResizeEw));
    assert!(!node.allows_self_pointer_hit_test());
    assert!(node.allows_child_pointer_hit_test());
    assert_eq!(UiCursor::ResizeEw.as_str(), "resize-ew");
}

#[test]
fn legacy_tree_nodes_default_missing_input_response_declarations() {
    let node = UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/child"));
    let mut legacy = serde_json::to_value(&node).expect("tree nodes serialize");
    let fields = legacy
        .as_object_mut()
        .expect("tree node serialization is an object");
    fields.remove("pointer_events");
    fields.remove("cursor");

    let restored: UiTreeNode = serde_json::from_value(legacy).expect("legacy nodes deserialize");

    assert_eq!(restored.pointer_events, UiPointerEvents::Auto);
    assert_eq!(restored.cursor, None);
}

#[test]
fn hit_path_derives_bubble_route_from_the_authoritative_root_to_leaf_path() {
    let root = UiNodeId::new(1);
    let container = UiNodeId::new(2);
    let target = UiNodeId::new(3);
    let query = UiHitTestQuery::new(UiPoint::new(12.0, 6.0));
    let path = UiHitPath::from_root_to_leaf(&query, vec![root, container, target]);

    assert_eq!(path.target, Some(target));
    assert_eq!(path.root_to_leaf, vec![root, container, target]);
    assert_eq!(path.bubble_route, vec![target, container, root]);
    assert!(path.has_consistent_route());
}

#[test]
#[should_panic(expected = "UiHitPath target must match the final root-to-leaf node")]
fn legacy_path_builder_rejects_a_target_that_disagrees_with_root_to_leaf() {
    let root = UiNodeId::new(1);
    let target = UiNodeId::new(3);
    let query = UiHitTestQuery::new(UiPoint::new(12.0, 6.0));
    let _ = UiHitPath::from_query(&query).with_route(Some(root), vec![root, target], vec![root]);
}

#[test]
#[should_panic(expected = "UiHitPath bubble route must be the reverse of root-to-leaf")]
fn legacy_path_builder_rejects_a_bubble_route_that_disagrees_with_root_to_leaf() {
    let root = UiNodeId::new(1);
    let target = UiNodeId::new(3);
    let query = UiHitTestQuery::new(UiPoint::new(12.0, 6.0));

    let _ = UiHitPath::from_query(&query).with_route(
        Some(target),
        vec![root, target],
        vec![root, target],
    );
}

#[test]
fn capture_is_a_distinct_dispatch_phase_from_overlay_preview() {
    assert_eq!(UiDispatchPhase::Capture.as_str(), "capture");
    assert_eq!(UiDispatchPhase::PreviewTunnel.as_str(), "preview_tunnel");
    assert!(UiDispatchPhase::Capture.is_hit_path_phase());
    assert!(UiDispatchPhase::Target.is_hit_path_phase());
    assert!(UiDispatchPhase::Bubble.is_hit_path_phase());
    assert!(!UiDispatchPhase::PreviewTunnel.is_hit_path_phase());
    assert_eq!(
        UiDispatchPhase::hit_path_sequence(),
        [
            UiDispatchPhase::Capture,
            UiDispatchPhase::Target,
            UiDispatchPhase::Bubble,
        ]
    );
}
