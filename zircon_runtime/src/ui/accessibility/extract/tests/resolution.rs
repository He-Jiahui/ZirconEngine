use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{accessibility::UiAccessibilityNode, event_ui::UiNodeId};

use super::refresh_node_ids;

#[test]
fn resolution_node_id_scratch_reuses_capacity_between_passes() {
    let mut nodes = BTreeMap::new();
    for value in 1..=8 {
        nodes.insert(UiNodeId::new(value), UiAccessibilityNode::default());
    }

    let mut node_ids = Vec::new();
    refresh_node_ids(&nodes, &mut node_ids);
    let retained_capacity = node_ids.capacity();
    assert_eq!(node_ids.len(), nodes.len());

    nodes.remove(&UiNodeId::new(8));
    refresh_node_ids(&nodes, &mut node_ids);

    assert_eq!(node_ids.len(), nodes.len());
    assert_eq!(node_ids.capacity(), retained_capacity);
}

#[test]
fn child_filter_uses_authoritative_map_membership_without_key_set_clone() {
    let source = include_str!("../resolution.rs");
    let filter_start = source
        .find("pub(super) fn filter_children(")
        .expect("child filter source");
    let filter_end = source[filter_start..]
        .find("fn refresh_node_ids(")
        .map(|offset| filter_start + offset)
        .expect("child filter boundary");
    let filter = &source[filter_start..filter_end];

    assert!(filter.contains("let included_node_ids = nodes.keys().copied().collect::<Vec<_>>();"));
    assert!(!filter.contains("let included: BTreeSet<_> = nodes.keys().copied().collect();"));
    let collector_start = source
        .find("fn collect_included_children(")
        .expect("child collector source");
    assert!(source[collector_start..].contains("included.contains_key(&node_id)"));
}
