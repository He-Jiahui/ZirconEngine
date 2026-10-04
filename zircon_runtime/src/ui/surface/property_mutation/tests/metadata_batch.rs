use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use super::*;

#[test]
fn virtual_window_metadata_batch_skips_unchanged_aliases_and_merges_dirty_once() {
    let node_id = UiNodeId::new(2);
    let mut tree = UiTree::new(UiTreeId::new("runtime.ui.virtual_window.batch"));
    tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root/table")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "Table".to_string(),
                attributes: BTreeMap::from([
                    ("viewport_start".to_string(), toml::Value::Integer(0)),
                    ("viewport_count".to_string(), toml::Value::Integer(4)),
                ]),
                ..UiTemplateNodeMetadata::default()
            },
        ),
    );

    // Consume root insertion's structural dirty state before testing the batch.
    let initial_dirty = tree.node(node_id).expect("node").dirty;
    assert!(initial_dirty.layout);
    assert!(initial_dirty.hit_test);
    assert!(initial_dirty.render);
    assert!(initial_dirty.input);
    tree.node_mut(node_id).expect("node").dirty = UiDirtyFlags::default();

    let batch = mutate_tree_metadata_properties(
        &mut tree,
        node_id,
        [
            ("viewport_start", UiValue::Int(2)),
            ("viewport_count", UiValue::Int(4)),
            ("visible_end", UiValue::Int(6)),
        ],
        UiBindingSourceKind::WidgetBehavior,
    )
    .expect("metadata batch should apply");

    assert_eq!(batch.changes.len(), 2);
    assert_eq!(batch.reflected_updates.len(), 2);
    assert!(batch
        .reflected_updates
        .iter()
        .all(|update| update.status == UiBindingUpdateStatus::Applied));
    assert!(batch.dirty.layout);
    assert!(batch.dirty.hit_test);
    assert!(batch.dirty.render);
    assert!(batch.dirty.input);
    assert!(batch.dirty.visible_range);
    assert!(!tree.node(node_id).expect("node").dirty.any());
    assert_eq!(
        tree.node(node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("viewport_start")),
        Some(&toml::Value::Integer(2))
    );
}
