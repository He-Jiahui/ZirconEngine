use super::{resolution_retained_capacity_budget, UiArrangedVisibilityIndex};
use crate::ui::surface::arranged_node_indices;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{UiArrangedNode, UiArrangedTree},
    tree::{UiInputPolicy, UiVisibility},
};

#[test]
fn hidden_ancestor_hides_visible_descendants() {
    let root = UiNodeId::new(10);
    let child = UiNodeId::new(2);
    let tree = arranged_tree(vec![
        arranged_node(child, Some(root), UiVisibility::Visible),
        arranged_node(root, None, UiVisibility::Hidden),
    ]);
    let index = visibility_index(&tree);

    assert!(!index.is_render_visible(root));
    assert!(!index.is_render_visible(child));
}

#[test]
fn self_hit_test_invisible_ancestor_remains_render_visible() {
    let root = UiNodeId::new(1);
    let child = UiNodeId::new(2);
    let tree = arranged_tree(vec![
        arranged_node(child, Some(root), UiVisibility::Visible),
        arranged_node(root, None, UiVisibility::SelfHitTestInvisible),
    ]);
    let index = visibility_index(&tree);

    assert!(index.is_render_visible(root));
    assert!(index.is_render_visible(child));
}

#[test]
fn missing_parent_fails_closed() {
    let node_id = UiNodeId::new(1);
    let tree = arranged_tree(vec![arranged_node(
        node_id,
        Some(UiNodeId::new(99)),
        UiVisibility::Visible,
    )]);

    assert!(!visibility_index(&tree).is_render_visible(node_id));
}

#[test]
fn parent_cycle_fails_closed() {
    let first = UiNodeId::new(1);
    let second = UiNodeId::new(2);
    let tree = arranged_tree(vec![
        arranged_node(first, Some(second), UiVisibility::Visible),
        arranged_node(second, Some(first), UiVisibility::Visible),
    ]);
    let index = visibility_index(&tree);

    assert!(!index.is_render_visible(first));
    assert!(!index.is_render_visible(second));
}

#[test]
fn resolution_scratch_reuses_warm_capacity_and_shrinks_to_current_tree() {
    let large_tree = chained_tree(128);
    let large_indices = arranged_node_indices(&large_tree);
    let mut index = UiArrangedVisibilityIndex::default();
    index.rebuild(&large_tree, &large_indices);
    let states_capacity = index.resolution_states.capacity();
    let states_pointer = index.resolution_states.as_ptr();
    let path_capacity = index.resolution_path.capacity();
    let path_pointer = index.resolution_path.as_ptr();

    index.rebuild(&large_tree, &large_indices);

    assert_eq!(index.resolution_states.capacity(), states_capacity);
    assert_eq!(index.resolution_states.as_ptr(), states_pointer);
    assert_eq!(index.resolution_path.capacity(), path_capacity);
    assert_eq!(index.resolution_path.as_ptr(), path_pointer);

    let slightly_smaller_tree = chained_tree(127);
    let slightly_smaller_indices = arranged_node_indices(&slightly_smaller_tree);
    index.rebuild(&slightly_smaller_tree, &slightly_smaller_indices);
    assert_eq!(index.resolution_states.as_ptr(), states_pointer);
    assert_eq!(index.resolution_path.as_ptr(), path_pointer);

    let small_tree = arranged_tree(vec![arranged_node(
        UiNodeId::new(1),
        None,
        UiVisibility::Visible,
    )]);
    let small_indices = arranged_node_indices(&small_tree);
    index.rebuild(&small_tree, &small_indices);

    let small_budget = resolution_retained_capacity_budget(small_tree.nodes.len());
    assert!(index.resolution_states.capacity() <= small_budget);
    assert!(index.resolution_path.capacity() <= small_budget);
    assert!(index.is_render_visible(UiNodeId::new(1)));

    let cloned = index.clone();
    assert!(cloned.resolution_states.is_empty());
    assert!(cloned.resolution_path.is_empty());
    assert_eq!(cloned, index);
}

#[test]
fn rebuild_reserves_published_node_id_capacity() {
    let tree = arranged_tree(vec![
        arranged_node(UiNodeId::new(10), None, UiVisibility::Visible),
        arranged_node(UiNodeId::new(20), None, UiVisibility::Visible),
        arranged_node(UiNodeId::new(30), None, UiVisibility::Visible),
    ]);
    let indices = arranged_node_indices(&tree);
    let mut index = UiArrangedVisibilityIndex::default();

    index.rebuild(&tree, &indices);

    assert!(index.node_ids.capacity() >= indices.len());
    assert!(index.is_render_visible(UiNodeId::new(20)));
}

#[test]
fn rebuild_publishes_sorted_ids_and_visibility_bits_in_one_index_pass() {
    let tree = arranged_tree(vec![
        arranged_node(UiNodeId::new(20), None, UiVisibility::Visible),
        arranged_node(UiNodeId::new(10), None, UiVisibility::Hidden),
    ]);
    let indices = arranged_node_indices(&tree);
    let mut index = UiArrangedVisibilityIndex::default();

    index.rebuild(&tree, &indices);

    assert_eq!(index.node_ids, vec![UiNodeId::new(10), UiNodeId::new(20)]);
    assert!(!index.is_render_visible(UiNodeId::new(10)));
    assert!(index.is_render_visible(UiNodeId::new(20)));
}

fn visibility_index(tree: &UiArrangedTree) -> UiArrangedVisibilityIndex {
    UiArrangedVisibilityIndex::from_arranged(tree, &arranged_node_indices(tree))
}

fn arranged_tree(nodes: Vec<UiArrangedNode>) -> UiArrangedTree {
    UiArrangedTree {
        tree_id: UiTreeId::new("arranged.visibility.index"),
        draw_order: nodes
            .iter()
            .map(|node| node.node_id)
            .collect::<Vec<_>>()
            .into(),
        nodes: nodes.into(),
        ..UiArrangedTree::default()
    }
}

fn chained_tree(count: u64) -> UiArrangedTree {
    let nodes = (0..count)
        .rev()
        .map(|index| {
            let node_id = UiNodeId::new(index);
            let parent = (index > 0).then(|| UiNodeId::new(index - 1));
            arranged_node(node_id, parent, UiVisibility::Visible)
        })
        .collect();
    arranged_tree(nodes)
}

fn arranged_node(
    node_id: UiNodeId,
    parent: Option<UiNodeId>,
    visibility: UiVisibility,
) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(format!("node/{}", node_id.0)),
        parent,
        children: Vec::new(),
        frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
        clip_frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
        z_index: 0,
        paint_order: node_id.0,
        visibility,
        input_policy: UiInputPolicy::Receive,
        pointer_events: Default::default(),
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: false,
        control_id: None,
        slot: None,
    }
}
