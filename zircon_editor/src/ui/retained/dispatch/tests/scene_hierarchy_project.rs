use super::*;

fn row(entity: EntityId, parent: Option<EntityId>, depth: u32) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity,
        parent,
        depth,
        display_name: format!("Node {entity}"),
        kind: "Entity".to_string(),
        subtree_hash: entity.wrapping_mul(17),
        active_in_hierarchy: true,
        has_children: false,
    }
}

#[test]
fn replace_builds_control_indexes_once_and_discards_stale_rows() {
    let mut state = SceneHierarchyProjectionState::default();
    let rows = [row(1, None, 0), row(2, Some(1), 1), row(3, Some(2), 2)];
    let controls = [String::from("row-a"), String::from("row-b")];
    let selected = BTreeSet::from([2]);

    state.replace(Some(7), Some(3), &rows, &controls, &selected);

    assert_eq!(state.control_for(1), Some("row-a"));
    assert_eq!(state.entity_for_control("row-b"), Some(2));
    assert!(state.contains_entity(3));
    assert!(state.is_selected(2));
    assert!(!state.contains_control("row-c"));

    let next_rows = [row(4, None, 0)];
    let next_controls = [String::from("row-z")];
    state.replace(
        Some(8),
        Some(4),
        &next_rows,
        &next_controls,
        &BTreeSet::new(),
    );

    assert!(!state.contains_entity(1));
    assert_eq!(state.control_for(4), Some("row-z"));
    assert_eq!(state.entity_for_control("row-a"), None);
    assert!(!state.is_selected(2));
}

#[test]
fn replace_shares_each_control_identifier_between_forward_and_reverse_indexes() {
    let mut state = SceneHierarchyProjectionState::default();
    let rows = [row(1, None, 0)];
    let controls = [String::from("row-a")];

    state.replace(None, None, &rows, &controls, &BTreeSet::new());

    let forward = state.controls_by_entity.get(&1).expect("forward control");
    let reverse = state
        .entities_by_control
        .keys()
        .next()
        .expect("reverse control");
    assert!(Arc::ptr_eq(forward, reverse));
}
