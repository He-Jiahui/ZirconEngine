#[test]
fn scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("components");

    for relative in ["mod.rs", "scene.rs"] {
        assert!(
            root.join(relative).exists(),
            "expected scene component module {relative} under {:?}",
            root
        );
    }

    let scene_root = root.parent().expect("scene directory exists");
    for relative in [
        "ecs/mod.rs",
        "ecs/archetype_id.rs",
        "ecs/bundle.rs",
        "ecs/component.rs",
        "ecs/component_id.rs",
        "ecs/component_registry.rs",
        "ecs/entity_location.rs",
        "ecs/entity_registry.rs",
        "ecs/internal_entity.rs",
        "ecs/resource.rs",
        "ecs/resource_id.rs",
        "ecs/resource_registry.rs",
        "ecs/schedule.rs",
        "ecs/scene_system_descriptor.rs",
        "ecs/scene_system_registry.rs",
        "ecs/storage/mod.rs",
        "ecs/storage_type.rs",
        "ecs/system_stage.rs",
    ] {
        assert!(
            scene_root.join(relative).exists(),
            "expected scene ECS module {relative} under {:?}",
            scene_root
        );
    }

    for relative in ["render_extract.rs", "viewport.rs", "gizmo.rs"] {
        assert!(
            !root.join(relative).exists(),
            "editor-owned scene authoring module {relative} should not live under {:?}",
            root
        );
    }
}

#[test]
fn world_property_access_moves_into_folder_backed_subtree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene")
        .join("world");

    assert!(
        root.join("property_access").join("mod.rs").exists(),
        "expected world property access to move into src/scene/world/property_access/mod.rs"
    );

    for relative in [
        "property_access/path_resolution.rs",
        "property_access/entries.rs",
        "property_access/read.rs",
        "property_access/write.rs",
        "property_access/value_conversion.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected world property-access module {relative} under {:?}",
            root
        );
    }

    assert!(
        !root.join("property_access.rs").exists(),
        "flat world property_access.rs should be replaced by a folder-backed subtree"
    );
}

#[test]
fn scene_render_extract_does_not_use_snapshot_adapter_for_frame_extract() {
    let render_extract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("render_extract")
            .join("mod.rs"),
    )
    .unwrap();

    assert!(
        !render_extract.contains("RenderFrameExtract::from_snapshot"),
        "scene render extract must populate RenderFrameExtract directly; from_snapshot is only for preview/test roundtrip adapters"
    );
}

#[test]
fn runtime_scene_exposes_neutral_world_inspection_surface() {
    let scene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scene");

    assert!(
        scene_root.join("inspection").join("mod.rs").exists(),
        "runtime scene should expose neutral world inspection under src/scene/inspection"
    );
    assert!(
        !scene_root.join("editor_projection").exists(),
        "runtime scene must not keep editor_projection as a production module"
    );

    for relative in [
        "mod.rs",
        "inspection/mod.rs",
        "inspection/hierarchy.rs",
        "inspection/field.rs",
        "inspection/snapshot.rs",
    ] {
        let source = std::fs::read_to_string(scene_root.join(relative)).unwrap();
        assert!(
            !source.contains("SceneEditor") && !source.contains("editor_projection"),
            "runtime scene inspection public surface must stay neutral in {relative}"
        );
    }
}

#[test]
fn scene_project_serialization_sources_do_not_store_editor_authoring_state() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in [
        "src/scene/world/world.rs",
        "src/scene/world/project_io.rs",
        "src/scene/dynamic_scene/document.rs",
        "src/scene/dynamic_scene/entity.rs",
        "src/scene/dynamic_scene/scene.rs",
        "src/scene/dynamic_scene/value.rs",
        "src/asset/assets/scene.rs",
    ] {
        let path = manifest_root.join(relative);
        let source = std::fs::read_to_string(&path).unwrap();

        for forbidden in [
            "selected",
            "selection",
            "selected_entity",
            "selected_node",
            "set_selected",
            "SceneViewportSettings",
            "SceneViewportTool",
            "TransformSpace",
            "GridMode",
            "ViewOrientation",
            "RenderOverlayExtract",
            "SelectionHighlightExtract",
            "SelectionAnchorExtract",
            "GridOverlayExtract",
            "HandleOverlayExtract",
            "SceneGizmoOverlayExtract",
            "SceneGizmoKind",
            "scene_gizmos",
            "selection_anchors",
            "active_camera_override",
            "camera_override",
            "preview_lighting",
            "preview_skybox",
            "display_mode",
        ] {
            assert!(
                !source.contains(forbidden),
                "runtime scene project serialization source {relative} must not store editor authoring state token {forbidden}"
            );
        }
    }
}

#[test]
fn scene_ecs_does_not_reintroduce_late_update_stage_or_compatibility_path() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in ["src/scene/ecs", "src/scene/module", "src/scene/world"] {
        assert_no_legacy_late_update_name(&manifest_root.join(relative));
    }
}

#[test]
fn component_storage_sparse_location_reads_value_and_ticks_from_single_entry() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
            .unwrap();
    let get_with_ticks_at_location_start = storage_source
        .find("pub fn get_with_ticks_at_location<T>")
        .expect("component storage get_with_ticks_at_location body should exist");
    let get_with_ticks_at_location_end = storage_source[get_with_ticks_at_location_start..]
        .find("\n    pub fn mark_changed(")
        .map(|offset| get_with_ticks_at_location_start + offset)
        .expect("component storage get_with_ticks_at_location body should end before mark_changed");
    let get_with_ticks_at_location_body =
        &storage_source[get_with_ticks_at_location_start..get_with_ticks_at_location_end];

    assert!(get_with_ticks_at_location_body.contains("self.sparse_components"));
    assert!(get_with_ticks_at_location_body.contains(".get(&location.component_id)?"));
    assert!(get_with_ticks_at_location_body.contains(".get_with_ticks(location.entity)"));
    assert!(!get_with_ticks_at_location_body
        .contains("let value = self.get::<T>(location.component_id, location.entity)?;"));
    assert!(!get_with_ticks_at_location_body
        .contains("let ticks = self.ticks(location.component_id, location.entity)?;"));

    let sparse_get_with_ticks_start = storage_source
        .find("impl SparseComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn get_with_ticks<T>")
                .map(|offset| start + offset)
        })
        .expect("sparse component get_with_ticks body should exist");
    let sparse_get_with_ticks_end = storage_source[sparse_get_with_ticks_start..]
        .find("\n    fn get_mut<T>")
        .map(|offset| sparse_get_with_ticks_start + offset)
        .expect("sparse component get_with_ticks body should end before get_mut");
    let sparse_get_with_ticks_body =
        &storage_source[sparse_get_with_ticks_start..sparse_get_with_ticks_end];

    assert!(sparse_get_with_ticks_body.contains("let entry = self.entries.get(&entity)?;"));
    assert!(sparse_get_with_ticks_body.contains("let value = entry.value.downcast_ref::<T>()?;"));
    assert!(sparse_get_with_ticks_body.contains("Some((value, entry.ticks))"));
}

#[test]
fn component_storage_get_mut_at_tick_uses_single_storage_dispatch() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
            .unwrap();
    let get_mut_at_tick_start = storage_source
        .find("pub fn get_mut_at_tick<T>")
        .expect("component storage get_mut_at_tick body should exist");
    let get_mut_at_tick_end = storage_source[get_mut_at_tick_start..]
        .find("\n    pub fn remove<T>")
        .map(|offset| get_mut_at_tick_start + offset)
        .expect("component storage get_mut_at_tick body should end before remove");
    let get_mut_at_tick_body = &storage_source[get_mut_at_tick_start..get_mut_at_tick_end];

    assert!(get_mut_at_tick_body.contains("match self.storage_types.get(&component_id).copied()?"));
    assert!(get_mut_at_tick_body
        .contains("let storage = self.table_components.get_mut(&component_id)?;"));
    assert!(get_mut_at_tick_body
        .contains("let storage = self.sparse_components.get_mut(&component_id)?;"));
    assert!(get_mut_at_tick_body.contains("storage.get_mut_at_tick(entity, tick)"));
    assert!(!get_mut_at_tick_body.contains("self.mark_changed(component_id, entity, tick);"));
    assert!(!get_mut_at_tick_body.contains("self.get_mut(component_id, entity)"));

    let table_get_mut_at_tick_start = storage_source
        .find("impl TableComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn get_mut_at_tick<T>")
                .map(|offset| start + offset)
        })
        .expect("table component get_mut_at_tick body should exist");
    let table_get_mut_at_tick_end = storage_source[table_get_mut_at_tick_start..]
        .find("\n    fn remove(")
        .map(|offset| table_get_mut_at_tick_start + offset)
        .expect("table component get_mut_at_tick body should end before remove");
    let table_get_mut_at_tick_body =
        &storage_source[table_get_mut_at_tick_start..table_get_mut_at_tick_end];

    assert!(table_get_mut_at_tick_body.contains("let row = self.rows.get(&entity).copied()?;"));
    assert!(table_get_mut_at_tick_body.contains("let entry = &mut self.entries[row];"));
    assert!(table_get_mut_at_tick_body.contains("entry.ticks.set_changed(tick);"));
    assert!(table_get_mut_at_tick_body.contains("entry.value.downcast_mut::<T>()"));

    let sparse_get_mut_at_tick_start = storage_source
        .find("impl SparseComponentStorage")
        .and_then(|start| {
            storage_source[start..]
                .find("fn get_mut_at_tick<T>")
                .map(|offset| start + offset)
        })
        .expect("sparse component get_mut_at_tick body should exist");
    let sparse_get_mut_at_tick_end = storage_source[sparse_get_mut_at_tick_start..]
        .find("\n    fn remove(")
        .map(|offset| sparse_get_mut_at_tick_start + offset)
        .expect("sparse component get_mut_at_tick body should end before remove");
    let sparse_get_mut_at_tick_body =
        &storage_source[sparse_get_mut_at_tick_start..sparse_get_mut_at_tick_end];

    assert!(sparse_get_mut_at_tick_body.contains("let entry = self.entries.get_mut(&entity)?;"));
    assert!(sparse_get_mut_at_tick_body.contains("entry.ticks.set_changed(tick);"));
    assert!(sparse_get_mut_at_tick_body.contains("entry.value.downcast_mut::<T>()"));
}

#[test]
fn component_storage_result_vectors_are_pre_sized_to_storage_count() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
            .unwrap();

    assert!(storage_source.contains("fn component_storage_count(&self) -> usize"));
    assert!(storage_source.contains("self.table_components.len() + self.sparse_components.len()"));
    assert!(storage_source
        .contains("let mut removed = Vec::with_capacity(self.component_storage_count());"));
    assert!(storage_source
        .contains("let mut component_ids = Vec::with_capacity(self.component_storage_count());"));
    assert!(!storage_source.contains("let mut removed = Vec::new();"));
    assert!(!storage_source.contains("let mut component_ids = Vec::new();"));
}

#[test]
fn component_storage_type_guards_use_entry_lookup() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn get<T>")
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
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn get_mut<T>")
        .map(|offset| get_start + offset)
        .expect("table component get body should end before get_mut");
    let get_body = &storage_source[get_start..get_end];

    assert!(get_body.contains("let row = *self.rows.get(&entity)?;"));
    assert!(get_body.contains("self.entries[row].value.downcast_ref::<T>()"));
    assert!(!get_body.contains("self.entries.get(*row)"));
}

#[test]
fn sparse_component_insert_uses_entry_lookup_for_replacement() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn get<T>")
        .map(|offset| insert_body_start + offset)
        .expect("sparse component insert body should end before get<T>");
    let insert_body = &storage_source[insert_body_start..insert_body_end];

    assert!(insert_body.contains("match self.entries.entry(entity)"));
    assert!(insert_body.contains("let entry = occupied.get_mut();"));
    assert!(insert_body.contains("std::mem::replace(&mut entry.value, value)"));
    assert!(insert_body.contains("vacant.insert(SparseEntry"));
    assert!(!insert_body.contains("self.entries.insert("));
    assert!(!insert_body.contains("self.entries.get_mut(&entity)"));
}

#[test]
fn table_component_ticks_uses_row_index_directly() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn mark_changed(")
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
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn len(")
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
    let storage_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/storage/component_storage.rs"))
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
        .find("\n    fn remove(")
        .map(|offset| get_mut_start + offset)
        .expect("table component get_mut body should end before remove");
    let get_mut_body = &storage_source[get_mut_start..get_mut_end];

    assert!(get_mut_body.contains("let row = self.rows.get(&entity).copied()?;"));
    assert!(get_mut_body.contains("self.entries[row].value.downcast_mut::<T>()"));
    assert!(!get_mut_body.contains("self.entries.get_mut(row)"));
}

fn assert_no_legacy_late_update_name(root: &std::path::Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            assert_no_legacy_late_update_name(&path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !source.contains("LateUpdate"),
            "scene ECS scheduling must not reintroduce LateUpdate aliases, shims, compatibility stages, or re-export bridges in {:?}",
            path
        );
    }
}
