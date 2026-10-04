use super::ChangedNodeAccumulator;

#[test]
fn changed_node_accumulator_promotes_only_for_distinct_nodes() {
    let mut accumulator = ChangedNodeAccumulator::default();
    let first = zircon_runtime_interface::ui::event_ui::UiNodeId::new(7);
    let second = zircon_runtime_interface::ui::event_ui::UiNodeId::new(3);

    accumulator.insert(first);
    accumulator.insert(first);
    assert_eq!(accumulator.single, Some(first));
    assert!(accumulator.many.is_none());

    accumulator.insert(second);
    assert!(accumulator.single.is_none());
    let many = accumulator
        .many
        .as_ref()
        .expect("distinct nodes promote to a set");
    assert_eq!(
        many.iter().copied().collect::<Vec<_>>(),
        vec![second, first]
    );
}

#[test]
fn single_component_state_dirty_path_avoids_the_batch_set_wrapper() {
    let source = include_str!("../state_invalidation.rs");
    let (_, single_node_path) = source
        .split_once("    pub(crate) fn mark_component_state_render_dirty(")
        .expect("the single-node state dirty entry point should remain surface-owned");
    let (single_node_path, _) = single_node_path
        .split_once("    pub(crate) fn mark_component_states_render_dirty(")
        .expect("the batch state dirty entry point should follow the single-node fast path");

    assert!(single_node_path.contains("node_state_can_affect_descendants(&self.tree, node_id)?"));
    assert!(single_node_path.contains("apply_runtime_state_style_subtree(node_id, true)?"));
    assert!(single_node_path.contains("apply_runtime_state_style_node(node_id, true)?"));
    assert!(single_node_path.contains("self.mark_node_dirty("));
    assert!(!single_node_path.contains("BTreeSet::from([node_id])"));
}
