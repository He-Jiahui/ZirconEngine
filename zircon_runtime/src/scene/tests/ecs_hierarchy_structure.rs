fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn removal_uses_indexed_identity_hierarchy_and_camera_boundaries() {
    let hierarchy = include_str!("../world/hierarchy.rs");
    let identity = include_str!("../world/identity.rs");
    let detached = include_str!("../world/transaction/detached_entity_batch.rs");
    let remove_entity =
        section_between(hierarchy, "pub fn remove_entity", "pub fn subtree_records");

    assert!(
        remove_entity.contains("-> SceneResult<()>") && remove_entity.contains("entity_dense_rows")
    );
    assert!(remove_entity.contains("self.direct_child_entity_ids(entity)"));
    assert!(remove_entity.contains("self.first_stable_camera_entity().unwrap_or(0)"));
    assert!(remove_entity.contains("self.remove_entity_from_dense_storage(entity)"));
    assert!(!remove_entity.contains("self.entities.remove"));
    assert!(!remove_entity.contains("self.stable_entity_ids().find"));
    assert!(identity.contains("self.entities.swap_remove(row)"));

    assert!(detached.contains("pub fn remove_entity_subtrees"));
    assert!(detached.contains("self.ensure_hierarchy_mutation_index_current()"));
    assert!(detached.contains("self.subtree_entity_ids(root)"));
    assert!(detached.contains("self.stable_query_order"));
    assert!(!detached.contains("self.entities.iter()"));
}

#[test]
fn mobility_validation_uses_direct_child_and_parent_scans() {
    let source = include_str!("../world/hierarchy.rs");
    let validate_mobility_change = section_between(
        source,
        "pub(super) fn validate_mobility_change",
        "fn ensure_transform_mutable",
    );

    assert!(
        validate_mobility_change.contains("for child in self.entities.iter().copied()")
            && validate_mobility_change.contains("if self.parent_of(child) != Some(entity)")
            && validate_mobility_change.contains("continue;")
            && validate_mobility_change
                .contains("if self.mobility(child) == Some(Mobility::Static)")
            && validate_mobility_change
                .contains("SceneError::DynamicMobilityWithStaticChildren { entity }")
            && !validate_mobility_change
                .contains(".filter(|child| self.parent_of(*child) == Some(entity))")
            && !validate_mobility_change.contains(".any(|child|"),
        "Dynamic mobility validation must scan children directly and stop at the first Static child"
    );
    assert!(
        validate_mobility_change.contains("if let Some(parent) = self.parent_of(entity)")
            && validate_mobility_change
                .contains("if self.mobility(parent) == Some(Mobility::Dynamic)")
            && validate_mobility_change.contains("SceneError::StaticMobilityUnderDynamicParent")
            && validate_mobility_change.contains("entity,")
            && validate_mobility_change.contains("parent,")
            && !validate_mobility_change.contains(".is_some_and("),
        "Static mobility validation must use a direct parent branch"
    );
}
