fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

fn method_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    section_between(source, start, end)
}

#[test]
fn component_storage_public_hot_paths_use_direct_storage_branches() {
    let store_source = include_str!("../ecs/storage/component_storage/store.rs");
    let insert_at_tick = method_between(store_source, "pub fn insert_at_tick<T>", "pub fn get<T>");
    let get = method_between(store_source, "pub fn get<T>", "pub fn get_mut<T>");
    let get_mut = method_between(
        store_source,
        "pub fn get_mut<T>",
        "pub fn get_mut_at_tick<T>",
    );
    let remove = method_between(store_source, "pub fn remove<T>", "pub fn contains");
    let contains = method_between(store_source, "pub fn contains", "pub fn ticks");
    let ticks = method_between(store_source, "pub fn ticks", "pub fn location");
    let location = method_between(store_source, "pub fn location", "pub fn get_table_row<T>");
    let get_table_row = method_between(
        store_source,
        "pub fn get_table_row<T>",
        "pub fn get_with_ticks_at_location<T>",
    );
    let get_with_ticks_at_location = method_between(
        store_source,
        "pub fn get_with_ticks_at_location<T>",
        "pub fn mark_changed",
    );

    assert!(
        insert_at_tick.contains("let Some(old) = old else")
            && !insert_at_tick.contains(".map(|old| downcast_component"),
        "component replacement must downcast the old value through a direct branch"
    );
    assert!(
        get.contains("let storage = self.table_components.get(&component_id)?;")
            && get.contains("let storage = self.sparse_components.get(&component_id)?;")
            && !get.contains(".and_then(|storage| storage.get(entity))"),
        "typed reads must fetch storage directly before component lookup"
    );
    assert!(
        get_mut.contains("let storage = self.table_components.get_mut(&component_id)?;")
            && get_mut.contains("let storage = self.sparse_components.get_mut(&component_id)?;")
            && !get_mut.contains(".and_then(|storage| storage.get_mut(entity))"),
        "mutable typed reads must fetch storage directly before component lookup"
    );
    assert!(
        remove.contains("let Some(storage) = self.table_components.get_mut(&component_id) else")
            && remove
                .contains("let Some(storage) = self.sparse_components.get_mut(&component_id) else")
            && remove.contains("let Some(removed) = removed else")
            && !remove.contains(".and_then(|storage| storage.remove(entity))")
            && !remove.contains(".map(|removed|"),
        "component removal must use direct missing-storage and missing-entity branches"
    );
    assert!(
        contains.contains("let Some(storage) = self.table_components.get(&component_id) else")
            && contains
                .contains("let Some(storage) = self.sparse_components.get(&component_id) else")
            && !contains.contains(".is_some_and("),
        "component membership checks must use direct storage branches"
    );
    assert!(
        ticks.contains("let storage = self.table_components.get(&component_id)?;")
            && ticks.contains("let storage = self.sparse_components.get(&component_id)?;")
            && !ticks.contains(".and_then(|storage| storage.ticks(entity))"),
        "component tick reads must fetch storage directly before tick lookup"
    );
    assert!(
        location.contains("if !storage.contains(entity)")
            && !location.contains(".then_some(ComponentStorageLocation"),
        "sparse component locations must use a direct contains branch"
    );
    assert!(
        get_table_row.contains("let storage = self.table_components.get(&component_id)?;")
            && !get_table_row.contains(".and_then(|storage| storage.get_row(row))"),
        "table-row lookup must fetch storage directly before row lookup"
    );
    assert!(
        get_with_ticks_at_location.contains("if entity != location.entity")
            && !get_with_ticks_at_location.contains(".then_some((value, ticks))"),
        "location-based table reads must validate entity identity through a direct branch"
    );
}

#[test]
fn sparse_component_storage_keeps_dense_rows_and_a_single_entity_index() {
    let sparse_source = include_str!("../ecs/storage/component_storage/sparse.rs");
    let component_results_source =
        include_str!("../ecs/storage/component_storage/component_results.rs");
    let sparse_storage = sparse_source;
    let remove = method_between(sparse_storage, "fn remove(", "fn contains(");

    assert!(
        sparse_storage.contains("entities: Vec<InternalEntity>")
            && sparse_storage.contains("entries: Vec<SparseEntry>")
            && sparse_storage.contains("indices: HashMap<InternalEntity, usize>")
            && sparse_storage.contains("let row = *self.indices.get(&entity)?;")
            && sparse_storage.contains("self.entries.get(row)")
            && sparse_storage.contains("self.entries.get_mut(row)")
            && sparse_storage.contains("for entity in self.entities.iter().copied()")
            && sparse_storage.contains("Some(entry.ticks)")
            && !sparse_storage.contains("HashMap<InternalEntity, SparseEntry>")
            && !sparse_storage.contains(".and_then(|entry|")
            && !sparse_storage.contains("remove(&entity).map(|entry|")
            && !sparse_storage.contains(".get(&entity).map(|entry| entry.ticks)"),
        "sparse component storage must keep its values in dense rows and address them through one entity index"
    );
    assert!(
        remove.contains("let row = self.indices.remove(&entity)?;")
            && remove.contains("let last_row = self.entries.len() - 1;")
            && remove.contains("let entry = self.entries.swap_remove(row);")
            && remove.contains("let removed_entity = self.entities.swap_remove(row);")
            && remove.contains("if row != last_row {")
            && remove.contains("self.indices.insert(swapped_entity, row);")
            && remove.contains("swapped_entity: None,"),
        "sparse removal must swap dense rows together and repair the moved entity index without exposing a table row"
    );
    assert!(
        component_results_source.contains("fn downcast_component<T>")
            && component_results_source.contains("match value.downcast::<T>()"),
        "component downcast must use a direct result branch"
    );
}

#[test]
fn component_storage_debug_storage_types_copy_is_pre_sized() {
    let source = include_str!("../ecs/storage/component_storage/store.rs");
    let debug_impl = section_between(
        source,
        "impl fmt::Debug for ComponentStorage {",
        "impl Clone for ComponentStorage {",
    );

    assert!(
        debug_impl.contains("let mut storage_types = Vec::with_capacity(self.storage_types.len())")
            && debug_impl.contains("for entry in &self.storage_types")
            && debug_impl.contains("storage_types.push(entry);")
            && debug_impl.contains("storage_types.sort_by_key(|(component_id, _)| **component_id)")
            && !debug_impl.contains("collect::<Vec<_>>()"),
        "component storage debug output must pre-size and copy storage types directly before sorting"
    );
}
