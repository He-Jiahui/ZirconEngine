use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    tree::UiTemplateNodeMetadata,
};

#[test]
fn owned_take_roundtrips_desired_identity() {
    let mut pool = UiSurfaceNodePool::default();
    let desired = test_node(2, "Button", "action.primary", "root/action");
    let component_ptr = desired
        .template_metadata
        .as_ref()
        .expect("test node metadata")
        .component
        .as_ptr();
    let control_id_ptr = desired
        .template_metadata
        .as_ref()
        .expect("test node metadata")
        .control_id
        .as_ref()
        .expect("test control id")
        .as_ptr();
    let path_ptr = desired.node_path.0.as_ptr();

    let (returned, pooled) = pool.take_owned(desired);

    assert!(pooled.is_none());
    assert_eq!(returned.node_id, UiNodeId::new(2));
    let metadata = returned
        .template_metadata
        .as_ref()
        .expect("returned metadata");
    assert_eq!(metadata.component.as_ptr(), component_ptr);
    assert_eq!(
        metadata.control_id.as_ref().unwrap().as_ptr(),
        control_id_ptr
    );
    assert_eq!(returned.node_path.0.as_ptr(), path_ptr);
}

#[test]
fn owned_take_reuses_matching_bucket() {
    let mut pool = UiSurfaceNodePool::default();
    assert!(pool.recycle(test_node(1, "Button", "action.primary", "root/action")));
    let desired = test_node(2, "Button", "action.primary", "root/action");
    let path_ptr = desired.node_path.0.as_ptr();

    let (returned, pooled) = pool.take_owned(desired);

    assert_eq!(returned.node_id, UiNodeId::new(2));
    assert_eq!(returned.node_path.0.as_ptr(), path_ptr);
    assert_eq!(
        pooled.expect("matching pooled node").node_id,
        UiNodeId::new(1)
    );
    assert_eq!(pool.resident_node_count(), 0);
    assert_eq!(pool.resident_bucket_count(), 0);
}

fn test_node(node_id: u64, component: &str, control_id: &str, node_path: &str) -> UiTreeNode {
    let mut node = UiTreeNode::new(UiNodeId::new(node_id), UiNodePath::new(node_path));
    node.template_metadata = Some(UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some(control_id.to_string()),
        ..UiTemplateNodeMetadata::default()
    });
    node
}
