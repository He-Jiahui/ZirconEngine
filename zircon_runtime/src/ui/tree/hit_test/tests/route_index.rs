use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::{UiPointerEvents, UiVisibility},
};

#[test]
fn deep_chain_builds_without_recursion() {
    const NODE_COUNT: usize = 4_096;
    let mut nodes = Vec::with_capacity(NODE_COUNT);
    for index in 0..NODE_COUNT {
        let node_id = UiNodeId::new((index + 1) as u64);
        let parent = (index > 0).then(|| UiNodeId::new(index as u64));
        let mut node = pointer_node(node_id, parent);
        if index + 1 < NODE_COUNT {
            node.children.push(UiNodeId::new((index + 2) as u64));
        }
        nodes.push(node);
    }
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.deep"),
        roots: vec![UiNodeId::new(1)].into(),
        draw_order: nodes
            .iter()
            .map(|node| node.node_id)
            .collect::<Vec<_>>()
            .into(),
        nodes: nodes.into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = arranged_tree
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect();
    let route_nodes = build_route_nodes(&arranged_tree, &node_indices);
    let leaf_id = UiNodeId::new(NODE_COUNT as u64);
    let entry = UiHitTestEntry {
        node_id: leaf_id,
        frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
        clip_frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
        z_index: 0,
        paint_order: 0,
        control_id: None,
        route_node_index: (NODE_COUNT - 1) as u32,
    };
    let grid = UiHitTestGrid {
        route_nodes,
        entries: vec![entry.clone()].into(),
        ..UiHitTestGrid::default()
    };

    let route = bubble_route_for_entry(&grid, &entry).expect("deep route must resolve");
    assert_eq!(route.len(), NODE_COUNT);
    assert_eq!(route.first(), Some(&leaf_id));
    assert_eq!(route.last(), Some(&UiNodeId::new(1)));
}

#[test]
fn missing_parent_and_cycle_fail_closed() {
    let missing_id = UiNodeId::new(1);
    let missing_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.missing"),
        roots: Vec::new().into(),
        nodes: vec![pointer_node(missing_id, Some(UiNodeId::new(99)))].into(),
        draw_order: vec![missing_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let missing_routes = build_route_nodes(&missing_tree, &BTreeMap::from([(missing_id, 0)]));
    assert!(!missing_routes[0].route_valid);

    let first_id = UiNodeId::new(10);
    let second_id = UiNodeId::new(11);
    let mut first = pointer_node(first_id, Some(second_id));
    first.children.push(second_id);
    let mut second = pointer_node(second_id, Some(first_id));
    second.children.push(first_id);
    let cycle_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.cycle"),
        roots: Vec::new().into(),
        nodes: vec![first, second].into(),
        draw_order: vec![first_id, second_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let cycle_routes = build_route_nodes(
        &cycle_tree,
        &BTreeMap::from([(first_id, 0), (second_id, 1)]),
    );
    assert!(cycle_routes.iter().all(|route| !route.route_valid));
}

#[test]
fn failed_component_does_not_poison_reused_traversal_scratch() {
    let invalid_id = UiNodeId::new(40);
    let valid_id = UiNodeId::new(41);
    let tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.reused-scratch"),
        roots: vec![valid_id].into(),
        nodes: vec![
            pointer_node(invalid_id, Some(UiNodeId::new(404))),
            pointer_node(valid_id, None),
        ]
        .into(),
        draw_order: vec![invalid_id, valid_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let routes = build_route_nodes(&tree, &BTreeMap::from([(invalid_id, 0), (valid_id, 1)]));

    assert!(!routes[0].route_valid);
    assert!(routes[1].route_valid);
    assert_eq!(routes[1].node_id, valid_id);
}

#[test]
fn route_publication_preallocates_node_table() {
    let nodes = vec![
        pointer_node(UiNodeId::new(50), None),
        pointer_node(UiNodeId::new(51), None),
        pointer_node(UiNodeId::new(52), None),
    ];
    let node_indices = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect();
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.capacity"),
        roots: vec![UiNodeId::new(50)].into(),
        nodes: nodes.into(),
        draw_order: Vec::new().into(),
        canvas_layers: Vec::new().into(),
    };

    let routes = build_route_nodes(&arranged_tree, &node_indices);

    assert_eq!(routes.len(), 3);
    assert!(routes.capacity() >= 3);
}

#[test]
fn input_patch_updates_descendant_route_semantics() {
    let parent_id = UiNodeId::new(20);
    let child_id = UiNodeId::new(21);
    let mut parent = pointer_node(parent_id, None);
    parent.children.push(child_id);
    parent.input_policy = UiInputPolicy::Receive;
    let child = pointer_node(child_id, Some(parent_id));
    let mut arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.input-patch"),
        roots: vec![parent_id].into(),
        nodes: vec![parent, child].into(),
        draw_order: vec![parent_id, child_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(parent_id, 0), (child_id, 1)]);
    let mut route_nodes = build_route_nodes(&arranged_tree, &node_indices);
    assert_eq!(
        route_nodes[1].effective_input_policy,
        UiInputPolicy::Receive
    );

    arranged_tree.nodes[0].input_policy = UiInputPolicy::Ignore;
    assert_eq!(
        patch_route_nodes(
            &mut route_nodes,
            &arranged_tree,
            &BTreeSet::from([parent_id, child_id]),
            &node_indices,
        ),
        Ok(true)
    );
    assert_eq!(route_nodes[1].effective_input_policy, UiInputPolicy::Ignore);
}

#[test]
fn input_patch_without_route_semantic_change_keeps_shared_allocation() {
    let node_id = UiNodeId::new(30);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.hit.route.noop-input-patch"),
        roots: vec![node_id].into(),
        nodes: vec![pointer_node(node_id, None)].into(),
        draw_order: vec![node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let node_indices = BTreeMap::from([(node_id, 0)]);
    let mut route_nodes = build_route_nodes(&arranged_tree, &node_indices);
    let shared = route_nodes.clone();

    assert_eq!(
        patch_route_nodes(
            &mut route_nodes,
            &arranged_tree,
            &BTreeSet::from([node_id]),
            &node_indices,
        ),
        Ok(false)
    );
    assert!(Arc::ptr_eq(&route_nodes, &shared));
}

fn pointer_node(node_id: UiNodeId, parent: Option<UiNodeId>) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(format!("root/{}", node_id.0)),
        parent,
        children: Vec::new(),
        frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
        clip_frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
        z_index: 0,
        paint_order: node_id.0,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Inherit,
        pointer_events: UiPointerEvents::Auto,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        clip_to_bounds: false,
        control_id: None,
        slot: None,
    }
}
