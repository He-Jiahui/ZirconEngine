use std::collections::{BTreeMap, HashSet};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiTree, UiTreeNode, UiVisibility},
};

use super::{resolve_detached_node, EffectiveHiddenIndex};

#[test]
fn effective_hidden_index_propagates_hidden_ancestors() {
    let mut tree = UiTree::new(UiTreeId::new("a11y.effective-hidden-index"));
    tree.insert_root(UiTreeNode::new(id(1), UiNodePath::new("root")));
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/hidden"))
            .with_visibility(UiVisibility::Collapsed),
    )
    .unwrap();
    tree.insert_child(
        id(2),
        UiTreeNode::new(id(3), UiNodePath::new("root/hidden/child")),
    )
    .unwrap();

    let index = EffectiveHiddenIndex::build(&tree, || Ok::<_, ()>(())).unwrap();

    assert!(!index.is_hidden(id(1)));
    assert!(index.is_hidden(id(2)));
    assert!(index.is_hidden(id(3)));
}

#[test]
fn detached_resolution_clears_reused_path_before_next_component() {
    let mut tree = UiTree::new(UiTreeId::new("a11y.detached-scratch"));
    tree.insert_root(UiTreeNode::new(id(1), UiNodePath::new("detached/visible")));
    tree.insert_root(
        UiTreeNode::new(id(2), UiNodePath::new("detached/hidden"))
            .with_visibility(UiVisibility::Collapsed),
    );
    tree.roots.clear();
    tree.node_mut(id(1)).expect("visible node exists").parent = Some(id(1));

    let mut hidden_by_node = BTreeMap::new();
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let mut check_deadline = || Ok::<(), ()>(());

    resolve_detached_node(
        &tree,
        id(1),
        &mut hidden_by_node,
        &mut path,
        &mut visited,
        &mut check_deadline,
    )
    .unwrap();
    assert_eq!(hidden_by_node.get(&id(1)), Some(&false));
    let retained_path_capacity = path.capacity();
    let retained_visited_capacity = visited.capacity();

    resolve_detached_node(
        &tree,
        id(2),
        &mut hidden_by_node,
        &mut path,
        &mut visited,
        &mut check_deadline,
    )
    .unwrap();

    assert_eq!(hidden_by_node.get(&id(1)), Some(&false));
    assert_eq!(hidden_by_node.get(&id(2)), Some(&true));
    assert_eq!(path.capacity(), retained_path_capacity);
    assert_eq!(visited.capacity(), retained_visited_capacity);
}

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}
