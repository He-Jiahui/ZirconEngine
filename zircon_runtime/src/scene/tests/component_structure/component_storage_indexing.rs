#[test]
fn component_storage_type_guards_use_entry_lookup() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/store.rs"),
    )
    .unwrap();

    assert!(storage_source.contains("use std::collections::hash_map::Entry;"));
    assert!(storage_source.contains("match self.storage_types.entry(component_id)"));
    assert!(storage_source.contains("match self.component_types.entry(component_id)"));
    assert!(storage_source.contains("Entry::Occupied(entry)"));
    assert!(storage_source.contains("Entry::Vacant(entry)"));
    assert!(!storage_source
        .contains("if let Some(existing) = self.storage_types.get(&component_id).copied()"));
    assert!(!storage_source
        .contains("if let Some(existing) = self.component_types.get(&component_id).copied()"));
}

#[test]
fn table_component_insert_uses_entry_lookup_for_row_index() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let insert_body_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn insert(")
                .map(|offset| start + offset)
        })
        .expect("table component insert body should exist");
    let insert_body_end = storage_source[insert_body_start..]
        .find("fn get<T>")
        .map(|offset| insert_body_start + offset)
        .expect("table component insert body should end before get<T>");
    let insert_body = &storage_source[insert_body_start..insert_body_end];

    assert!(insert_body.contains("match self.rows.entry(entity)"));
    assert!(insert_body.contains("let row = *entry.get();"));
    assert!(insert_body.contains("entry.insert(row);"));
    assert!(!insert_body.contains("if let Some(row) = self.rows.get(&entity).copied()"));
    assert!(!insert_body.contains("self.rows.insert(entity, row);"));
}

#[test]
fn table_component_get_uses_row_index_directly() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let get_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn get<T>")
                .map(|offset| start + offset)
        })
        .expect("table component get body should exist");
    let get_end = storage_source[get_start..]
        .find("fn get_mut<T>")
        .map(|offset| get_start + offset)
        .expect("table component get body should end before get_mut");
    let get_body = &storage_source[get_start..get_end];

    assert!(get_body.contains("let row = *self.rows.get(&entity)?;"));
    assert!(get_body.contains("self.entries[row].value.downcast_ref::<T>()"));
    assert!(!get_body.contains("self.entries.get(*row)"));
}

#[test]
fn sparse_component_insert_keeps_dense_rows_indexed_by_entity() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/sparse.rs"),
    )
    .unwrap();
    let insert_body_start = storage_source
        .find("impl SparseComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn insert(")
                .map(|offset| start + offset)
        })
        .expect("sparse component insert body should exist");
    let insert_body_end = storage_source[insert_body_start..]
        .find("fn get<T>")
        .map(|offset| insert_body_start + offset)
        .expect("sparse component insert body should end before get<T>");
    let insert_body = &storage_source[insert_body_start..insert_body_end];

    assert!(storage_source.contains("entities: Vec<InternalEntity>"));
    assert!(storage_source.contains("entries: Vec<SparseEntry>"));
    assert!(storage_source.contains("indices: HashMap<InternalEntity, usize>"));
    assert!(insert_body.contains("if let Some(row) = self.indices.get(&entity).copied()"));
    assert!(insert_body.contains("let entry = &mut self.entries[row];"));
    assert!(insert_body.contains("std::mem::replace(&mut entry.value, value)"));
    assert!(insert_body.contains("let row = self.entries.len();"));
    assert!(insert_body.contains("self.entries.push(SparseEntry"));
    assert!(insert_body.contains("self.entities.push(entity);"));
    assert!(insert_body.contains("self.indices.insert(entity, row);"));
    assert!(!storage_source.contains("HashMap<InternalEntity, SparseEntry>"));
    assert!(!insert_body.contains("match self.entries.entry(entity)"));
}

#[test]
fn table_component_ticks_uses_row_index_directly() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let ticks_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn ticks(")
                .map(|offset| start + offset)
        })
        .expect("table component ticks body should exist");
    let ticks_end = storage_source[ticks_start..]
        .find("fn mark_changed(")
        .map(|offset| ticks_start + offset)
        .expect("table component ticks body should end before mark_changed");
    let ticks_body = &storage_source[ticks_start..ticks_end];

    assert!(ticks_body.contains("let row = *self.rows.get(&entity)?;"));
    assert!(ticks_body.contains("Some(self.entries[row].ticks)"));
    assert!(!ticks_body.contains("self.entries.get(*row)"));
}

#[test]
fn table_component_mark_changed_uses_row_index_directly() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let mark_changed_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn mark_changed(")
                .map(|offset| start + offset)
        })
        .expect("table component mark_changed body should exist");
    let mark_changed_end = storage_source[mark_changed_start..]
        .find("fn len(")
        .map(|offset| mark_changed_start + offset)
        .expect("table component mark_changed body should end before len");
    let mark_changed_body = &storage_source[mark_changed_start..mark_changed_end];

    assert!(mark_changed_body.contains("self.entries[row].ticks.set_changed(tick);"));
    assert!(mark_changed_body.contains("let Some(row) = self.rows.get(&entity).copied() else"));
    assert!(!mark_changed_body.contains("self.entries.get_mut(row)"));
}

#[test]
fn table_component_get_mut_uses_row_index_directly() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/table.rs"),
    )
    .unwrap();
    let get_mut_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn get_mut<T>")
                .map(|offset| start + offset)
        })
        .expect("table component get_mut body should exist");
    let get_mut_end = storage_source[get_mut_start..]
        .find("fn remove(")
        .map(|offset| get_mut_start + offset)
        .expect("table component get_mut body should end before remove");
    let get_mut_body = &storage_source[get_mut_start..get_mut_end];

    assert!(get_mut_body.contains("let row = self.rows.get(&entity).copied()?;"));
    assert!(get_mut_body.contains("self.entries[row].value.downcast_mut::<T>()"));
    assert!(!get_mut_body.contains("self.entries.get_mut(row)"));
}
