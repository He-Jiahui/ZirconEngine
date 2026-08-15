#[test]
fn component_storage_type_guards_use_entry_lookup() {
    let storage_source = include_str!("../../ecs/storage/component_storage/store.rs");

    assert!(storage_source.contains("use std::collections::hash_map::Entry;"));
    assert!(storage_source.contains("match self.storage_types.entry(component_id)"));
    assert!(storage_source.contains("match self.component_types.entry(component_id)"));
    assert!(storage_source.contains("Entry::Occupied(entry)"));
    assert!(storage_source.contains("Entry::Vacant(entry)"));
}

#[test]
fn archetype_table_compiles_sorted_component_ids_into_column_slots() {
    let table_source = include_str!("../../ecs/archetype/table/table.rs");

    assert!(table_source.contains("columns: Vec<(ComponentId, ArchetypeColumn)>"));
    assert!(
        table_source.contains("columns.sort_unstable_by_key(|(component_id, _)| *component_id)")
    );
    assert!(table_source.contains("columns.dedup_by_key(|(component_id, _)| *component_id)"));
    assert!(
        table_source.contains("binary_search_by_key(&component_id, |(candidate, _)| *candidate)")
    );
    assert!(table_source.contains("self.columns.get(column_slot)?.1.get::<T>(row)"));
    assert!(table_source.contains("self.columns.get_mut(column_slot)?"));
    assert!(!table_source.contains("HashMap<ComponentId, ArchetypeColumn>"));
}

#[test]
fn archetype_table_row_take_repairs_all_columns_by_the_same_swap_row() {
    let table_source = include_str!("../../ecs/archetype/table/table.rs");
    let take_row = table_source
        .split("pub(crate) fn take_row(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn for_each_component<T>").next())
        .expect("archetype table take_row body");

    assert!(take_row.contains("for (component_id, column) in &mut self.columns"));
    assert!(take_row.contains("column.take(row)"));
    assert!(take_row.contains("self.entities.swap_remove(row)"));
    assert!(take_row.contains("let swapped_entity = (row < self.entities.len())"));
    assert!(take_row.contains("ArchetypeTakenRow::new(entity, swapped_entity, components)"));
}

#[test]
fn sparse_component_storage_keeps_dense_rows_indexed_by_entity() {
    let sparse_source = include_str!("../../ecs/storage/component_storage/sparse.rs");
    let insert = sparse_source
        .split("fn insert(")
        .nth(1)
        .and_then(|body| body.split("fn get<T>").next())
        .expect("sparse insert body");

    assert!(sparse_source.contains("entities: Vec<InternalEntity>"));
    assert!(sparse_source.contains("entries: Vec<SparseEntry>"));
    assert!(sparse_source.contains("sparse_rows: Vec<Option<SparseRowLocation>>"));
    assert!(insert.contains("if let Some(row) = self.dense_row(entity)"));
    assert!(insert.contains("self.entries.push(SparseEntry"));
    assert!(insert.contains("self.entities.push(entity);"));
    assert!(insert.contains("self.set_sparse_row(entity, row);"));
    assert!(!sparse_source.contains("HashMap<InternalEntity, usize>"));
    assert!(!sparse_source.contains("HashMap<InternalEntity, SparseEntry>"));
}
