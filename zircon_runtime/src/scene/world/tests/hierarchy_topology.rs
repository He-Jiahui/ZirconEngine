use super::HierarchyTopology;

#[test]
fn roots_preserve_stable_order_across_reparent_and_removal() {
    let mut topology = HierarchyTopology::default();
    topology.update_parent(10, 1, None, None);
    topology.update_parent(20, 2, None, None);
    topology.update_parent(30, 3, None, None);
    assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 20, 30]);

    topology.update_parent(20, 2, None, Some(10));
    assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 30]);
    assert_eq!(topology.children_of(10).collect::<Vec<_>>(), vec![20]);

    topology.update_parent(20, 2, Some(10), None);
    assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 20, 30]);

    topology.remove_entity(20, 2, None);
    assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 30]);
}

#[test]
fn parent_projection_tracks_structural_updates_and_rebuilds() {
    let mut topology = HierarchyTopology::default();
    topology.update_parent(10, 1, None, None);
    topology.update_parent(20, 2, None, Some(10));
    assert_eq!(topology.parent_of(10), None);
    assert_eq!(topology.parent_of(20), Some(10));

    topology.update_parent(20, 2, Some(10), None);
    assert_eq!(topology.parent_of(20), None);

    topology.mark_dirty();
    topology.rebuild([(10, 1, None), (20, 2, Some(10))]);
    assert_eq!(topology.parent_of(20), Some(10));

    topology.remove_entity(20, 2, Some(10));
    topology.mark_current();
    assert_eq!(topology.parent_of(20), None);
}

#[test]
fn missing_parent_projection_row_forces_source_rebuild() {
    let mut topology = HierarchyTopology::default();
    topology.update_parent(10, 1, None, None);
    topology.update_parent(20, 2, None, Some(10));
    topology.parent_by_entity.remove(&20);

    assert!(!topology.is_current_for_entity_count(2));
    assert!(topology.needs_source_rebuild(2));
}
