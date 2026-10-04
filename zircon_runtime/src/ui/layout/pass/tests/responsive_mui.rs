use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    layout::UiSize,
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
};

use super::{apply_mui_responsive_layout, MuiResponsiveCandidates};

fn node(id: u64, component: &str, attributes: &[(&str, Value)]) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("root/{id}")))
        .with_template_metadata(UiTemplateNodeMetadata {
            component: component.to_string(),
            attributes: attributes
                .iter()
                .map(|(name, value)| ((*name).to_string(), value.clone()))
                .collect(),
            ..UiTemplateNodeMetadata::default()
        })
}

#[test]
fn full_pass_candidates_exclude_non_responsive_template_nodes() {
    let mut tree = UiTree::default();
    tree.nodes.insert(UiNodeId::new(1), node(1, "Button", &[]));
    tree.nodes.insert(
        UiNodeId::new(2),
        node(
            2,
            "UseMediaQuery",
            &[("query", Value::String("(min-width: 600px)".into()))],
        ),
    );
    tree.nodes.insert(
        UiNodeId::new(3),
        node(3, "Box", &[("display", Value::String("none".into()))]),
    );
    tree.nodes.insert(
        UiNodeId::new(4),
        node(4, "Grid", &[("container", Value::Boolean(true))]),
    );

    let candidates = MuiResponsiveCandidates::for_tree(&tree);

    assert_eq!(
        candidates.media_query_node_ids,
        [UiNodeId::new(2)].into_iter().collect()
    );
    assert_eq!(
        candidates.visibility_node_ids,
        [UiNodeId::new(3)].into_iter().collect()
    );
    assert_eq!(
        candidates.container_node_ids,
        [UiNodeId::new(4)].into_iter().collect()
    );
    assert_eq!(
        candidates.implicit_grid_parent_ids,
        [UiNodeId::new(4)].into_iter().collect()
    );
}

#[test]
fn candidate_patch_tracks_same_cardinality_metadata_changes() {
    let node_id = UiNodeId::new(1);
    let mut tree = UiTree::default();
    tree.nodes.insert(node_id, node(1, "Button", &[]));
    let mut candidates = MuiResponsiveCandidates::for_tree(&tree);
    assert!(candidates.media_query_node_ids.is_empty());

    tree.nodes.insert(
        node_id,
        node(
            1,
            "UseMediaQuery",
            &[("query", Value::String("(min-width: 600px)".into()))],
        ),
    );
    candidates.patch_nodes(&tree, &[node_id].into_iter().collect());

    assert_eq!(
        candidates.media_query_node_ids,
        [node_id].into_iter().collect()
    );
}

#[test]
fn unchanged_responsive_values_do_not_create_mutation_candidates() {
    let node_id = UiNodeId::new(1);
    let mut tree = UiTree::default();
    tree.nodes.insert(
        node_id,
        node(
            1,
            "UseMediaQuery",
            &[
                ("query", Value::String("(min-width: 600px)".into())),
                ("matches", Value::Boolean(true)),
            ],
        ),
    );
    tree.clear_pending_mutation_node_ids();

    apply_mui_responsive_layout(&mut tree, UiSize::new(800.0, 600.0))
        .expect("stable responsive pass");

    assert!(tree.pending_mutation_node_ids().is_empty());
}
