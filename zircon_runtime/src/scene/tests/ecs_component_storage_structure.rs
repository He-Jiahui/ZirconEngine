#[test]
fn dense_and_sparse_component_values_have_exactly_one_storage_owner() {
    let component_storage = include_str!("../ecs/storage/component_storage/store.rs");
    let archetype_record = include_str!("../ecs/archetype/record.rs");
    let archetype_table = include_str!("../ecs/archetype/table/table.rs");

    assert!(component_storage
        .contains("sparse_components: HashMap<ComponentId, SparseComponentStorage>"));
    assert!(!component_storage.contains("table_components:"));
    assert!(!component_storage.contains("TableComponentStorage"));
    assert!(archetype_record.contains("table: ArchetypeTable"));
    assert!(archetype_table.contains("entities: Vec<EntityId>"));
    assert!(archetype_table.contains("columns: Vec<(ComponentId, ArchetypeColumn)>"));
}

#[test]
fn sparse_component_storage_keeps_dense_rows_and_a_single_entity_index() {
    let sparse = include_str!("../ecs/storage/component_storage/sparse.rs");
    let remove = sparse
        .split("fn remove(")
        .nth(1)
        .and_then(|body| body.split("fn contains(").next())
        .expect("sparse remove body");

    assert!(sparse.contains("entities: Vec<InternalEntity>"));
    assert!(sparse.contains("entries: Vec<SparseEntry>"));
    assert!(sparse.contains("sparse_rows: Vec<Option<SparseRowLocation>>"));
    assert!(sparse.contains("generation: u32"));
    assert!(sparse.contains("dense_row: usize"));
    assert!(!sparse.contains("HashMap<InternalEntity, usize>"));
    assert!(!sparse.contains("HashMap<InternalEntity, SparseEntry>"));
    assert!(remove.contains("let row = self.remove_sparse_row(entity)?;"));
    assert!(remove.contains("let entry = self.entries.swap_remove(row);"));
    assert!(remove.contains("self.entities.swap_remove(row);"));
    assert!(remove.contains("self.set_sparse_row(swapped_entity, row);"));
    assert!(!remove.contains("swapped_entity: None"));
}

#[test]
fn archetype_table_slot_access_avoids_component_map_lookup_per_row() {
    let table = include_str!("../ecs/archetype/table/table.rs");
    let get_by_slot = table
        .split("pub(crate) fn get_by_slot<T>")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn component_ticks_by_slot").next())
        .expect("column-slot read body");

    assert!(get_by_slot.contains("self.columns.get(column_slot)?.1.get::<T>(row)"));
    assert!(!get_by_slot.contains("column_slot("));
    assert!(!get_by_slot.contains("binary_search"));
}
