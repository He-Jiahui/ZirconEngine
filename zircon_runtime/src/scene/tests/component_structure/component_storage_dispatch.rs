#[test]
fn component_storage_stays_sparse_only_after_archetype_table_cutover() {
    let store_source = include_str!("../../ecs/storage/component_storage/store.rs");
    let storage_mod_source = include_str!("../../ecs/storage/component_storage/mod.rs");

    assert!(
        store_source.contains("sparse_components: HashMap<ComponentId, SparseComponentStorage>")
    );
    assert!(store_source.contains("StorageError::TableOwnedByArchetype"));
    assert!(store_source.contains("self.sparse_components.get(&component_id)?.get(entity)"));
    assert!(
        store_source.contains("self.sparse_components.get_mut(&component_id)?.get_mut(entity)")
    );
    for forbidden in [
        "TableComponentStorage",
        "table_components:",
        "pub fn get_table_row",
        "pub fn remove_entity(",
        "component_ids_for_entity_by_storage",
        "for_each_table_component",
    ] {
        assert!(
            !store_source.contains(forbidden),
            "ComponentStorage must not regain dense-table authority `{forbidden}`"
        );
    }
    assert!(!storage_mod_source.contains("mod table;"));
}

#[test]
fn component_storage_sparse_location_reads_value_and_ticks_from_single_entry() {
    let store_source = include_str!("../../ecs/storage/component_storage/store.rs");
    let sparse_source = include_str!("../../ecs/storage/component_storage/sparse.rs");
    let get_with_ticks_at_location = store_source
        .split("pub fn get_with_ticks_at_location<T>")
        .nth(1)
        .and_then(|body| body.split("pub fn mark_changed").next())
        .expect("location read body");

    assert!(get_with_ticks_at_location.contains("self.sparse_components"));
    assert!(get_with_ticks_at_location.contains(".get(&location.component_id)?"));
    assert!(get_with_ticks_at_location.contains(".get_with_ticks(location.entity)"));
    assert!(!get_with_ticks_at_location.contains("self.get::<T>"));
    assert!(!get_with_ticks_at_location.contains("self.ticks("));

    let sparse_get_with_ticks = sparse_source
        .split("fn get_with_ticks<T>")
        .nth(1)
        .and_then(|body| body.split("fn get_mut<T>").next())
        .expect("sparse location read body");
    assert!(sparse_get_with_ticks.contains("let entry = self.entry(entity)?;"));
    assert!(sparse_get_with_ticks.contains("entry.value.downcast_ref::<T>()?"));
    assert!(sparse_get_with_ticks.contains("Some((value, entry.ticks))"));
}

#[test]
fn component_storage_sparse_row_extraction_is_signature_directed() {
    let store_source = include_str!("../../ecs/storage/component_storage/store.rs");
    let extract = store_source
        .split("pub(crate) fn extract_entity_rows")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn insert_transferred_row").next())
        .expect("sparse transfer extraction body");
    let remove = store_source
        .split("pub(crate) fn remove_entity_components")
        .nth(1)
        .and_then(|body| body.split("pub fn storage_type").next())
        .expect("sparse component removal body");

    for body in [extract, remove] {
        assert!(body.contains("for component_id in component_ids"));
        assert!(body.contains("self.sparse_components"));
        assert!(!body.contains("values_mut()"));
        assert!(!body.contains("table_components"));
    }
}
