#[test]
fn component_storage_sparse_location_reads_value_and_ticks_from_single_entry() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let store_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/store.rs"),
    )
    .unwrap();
    let sparse_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/sparse.rs"),
    )
    .unwrap();
    let get_with_ticks_at_location_start = store_source
        .find("pub fn get_with_ticks_at_location<T>")
        .expect("component storage get_with_ticks_at_location body should exist");
    let get_with_ticks_at_location_end = store_source[get_with_ticks_at_location_start..]
        .find("\n    pub fn mark_changed(")
        .map(|offset| get_with_ticks_at_location_start + offset)
        .expect("component storage get_with_ticks_at_location body should end before mark_changed");
    let get_with_ticks_at_location_body =
        &store_source[get_with_ticks_at_location_start..get_with_ticks_at_location_end];

    assert!(get_with_ticks_at_location_body.contains("self.sparse_components"));
    assert!(get_with_ticks_at_location_body.contains(".get(&location.component_id)?"));
    assert!(get_with_ticks_at_location_body.contains(".get_with_ticks(location.entity)"));
    assert!(
        !get_with_ticks_at_location_body
            .contains("let value = self.get::<T>(location.component_id, location.entity)?;")
    );
    assert!(
        !get_with_ticks_at_location_body
            .contains("let ticks = self.ticks(location.component_id, location.entity)?;")
    );

    let sparse_get_with_ticks_start = sparse_source
        .find("impl SparseComponentStorage")
        .and_then(|start| {
            sparse_source[start..]
                .find("fn get_with_ticks<T>")
                .map(|offset| start + offset)
        })
        .expect("sparse component get_with_ticks body should exist");
    let sparse_get_with_ticks_end = sparse_source[sparse_get_with_ticks_start..]
        .find("fn get_mut<T>")
        .map(|offset| sparse_get_with_ticks_start + offset)
        .expect("sparse component get_with_ticks body should end before get_mut");
    let sparse_get_with_ticks_body =
        &sparse_source[sparse_get_with_ticks_start..sparse_get_with_ticks_end];

    assert!(sparse_get_with_ticks_body.contains("let entry = self.entries.get(&entity)?;"));
    assert!(sparse_get_with_ticks_body.contains("let value = entry.value.downcast_ref::<T>()?;"));
    assert!(sparse_get_with_ticks_body.contains("Some((value, entry.ticks))"));
}

#[test]
fn component_storage_get_mut_at_tick_uses_single_storage_dispatch() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let store_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/store.rs"),
    )
    .unwrap();
    let table_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let sparse_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/sparse.rs"),
    )
    .unwrap();
    let get_mut_at_tick_start = store_source
        .find("pub fn get_mut_at_tick<T>")
        .expect("component storage get_mut_at_tick body should exist");
    let get_mut_at_tick_end = store_source[get_mut_at_tick_start..]
        .find("\n    pub fn remove<T>")
        .map(|offset| get_mut_at_tick_start + offset)
        .expect("component storage get_mut_at_tick body should end before remove");
    let get_mut_at_tick_body = &store_source[get_mut_at_tick_start..get_mut_at_tick_end];

    assert!(get_mut_at_tick_body.contains("match self.storage_types.get(&component_id).copied()?"));
    assert!(
        get_mut_at_tick_body
            .contains("let storage = self.table_components.get_mut(&component_id)?;")
    );
    assert!(
        get_mut_at_tick_body
            .contains("let storage = self.sparse_components.get_mut(&component_id)?;")
    );
    assert!(get_mut_at_tick_body.contains("storage.get_mut_at_tick(entity, tick)"));
    assert!(!get_mut_at_tick_body.contains("self.mark_changed(component_id, entity, tick);"));
    assert!(!get_mut_at_tick_body.contains("self.get_mut(component_id, entity)"));

    let table_get_mut_at_tick_start = table_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            table_source[start..]
                .find("fn get_mut_at_tick<T>")
                .map(|offset| start + offset)
        })
        .expect("table component get_mut_at_tick body should exist");
    let table_get_mut_at_tick_end = table_source[table_get_mut_at_tick_start..]
        .find("fn remove(")
        .map(|offset| table_get_mut_at_tick_start + offset)
        .expect("table component get_mut_at_tick body should end before remove");
    let table_get_mut_at_tick_body =
        &table_source[table_get_mut_at_tick_start..table_get_mut_at_tick_end];

    assert!(table_get_mut_at_tick_body.contains("let row = self.rows.get(&entity).copied()?;"));
    assert!(table_get_mut_at_tick_body.contains("let entry = &mut self.entries[row];"));
    assert!(table_get_mut_at_tick_body.contains("entry.ticks.set_changed(tick);"));
    assert!(table_get_mut_at_tick_body.contains("entry.value.downcast_mut::<T>()"));

    let sparse_get_mut_at_tick_start = sparse_source
        .find("impl SparseComponentStorage")
        .and_then(|start| {
            sparse_source[start..]
                .find("fn get_mut_at_tick<T>")
                .map(|offset| start + offset)
        })
        .expect("sparse component get_mut_at_tick body should exist");
    let sparse_get_mut_at_tick_end = sparse_source[sparse_get_mut_at_tick_start..]
        .find("fn remove(")
        .map(|offset| sparse_get_mut_at_tick_start + offset)
        .expect("sparse component get_mut_at_tick body should end before remove");
    let sparse_get_mut_at_tick_body =
        &sparse_source[sparse_get_mut_at_tick_start..sparse_get_mut_at_tick_end];

    assert!(sparse_get_mut_at_tick_body.contains("let entry = self.entries.get_mut(&entity)?;"));
    assert!(sparse_get_mut_at_tick_body.contains("entry.ticks.set_changed(tick);"));
    assert!(sparse_get_mut_at_tick_body.contains("entry.value.downcast_mut::<T>()"));
}

#[test]
fn component_storage_result_vectors_are_pre_sized_to_storage_count() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let store_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/store.rs"),
    )
    .unwrap();
    let component_results_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/component_results.rs"),
    )
    .unwrap();
    let remove_entity_start = store_source
        .find("pub fn remove_entity")
        .expect("component storage remove_entity body should exist");
    let remove_entity_end = store_source[remove_entity_start..]
        .find("\n    pub(crate) fn component_ids_for_entity")
        .map(|offset| remove_entity_start + offset)
        .expect("component storage remove_entity body should end before component_ids_for_entity");
    let remove_entity_body = &store_source[remove_entity_start..remove_entity_end];

    let component_ids_start = store_source
        .find("pub(crate) fn component_ids_for_entity")
        .expect("component storage component_ids_for_entity body should exist");
    let component_ids_end = store_source[component_ids_start..]
        .find("\n    fn component_storage_count")
        .map(|offset| component_ids_start + offset)
        .expect("component storage component_ids_for_entity body should end before helper methods");
    let component_ids_body = &store_source[component_ids_start..component_ids_end];

    assert!(store_source.contains("fn component_storage_count(&self) -> usize"));
    assert!(store_source.contains("self.table_components.len() + self.sparse_components.len()"));
    assert!(
        remove_entity_body
            .contains("let mut removed = Vec::with_capacity(self.component_storage_count());")
    );
    assert!(
        component_ids_body.contains(
            "let mut component_ids = Vec::with_capacity(self.component_storage_count());"
        )
    );
    assert!(remove_entity_body.contains("sort_component_ids_if_needed(&mut removed);"));
    assert!(component_ids_body.contains("sort_component_ids_if_needed(&mut component_ids);"));
    assert!(
        component_results_source.contains("fn sort_component_ids_if_needed(")
            && component_results_source.contains("component_ids: &mut [ComponentId]")
    );
    assert!(component_results_source.contains("if component_ids.len() > 1"));
    assert!(component_results_source.contains("component_ids.sort_unstable();"));
    assert!(!store_source.contains("let mut removed = Vec::new();"));
    assert!(!store_source.contains("let mut component_ids = Vec::new();"));
    assert!(!remove_entity_body.contains("removed.sort_unstable();"));
    assert!(!component_ids_body.contains("component_ids.sort_unstable();"));
}
