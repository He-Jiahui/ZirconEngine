use std::sync::{Arc, Mutex};

use crate::scene::components::{CameraComponent, Name};
use crate::scene::ecs::{ArchetypeId, Component, EntityLocation, EntityRegistry, StorageType};
use crate::scene::{NodeKind, World};

#[derive(Debug, PartialEq, Eq)]
struct TestComponent(&'static str);

#[derive(Debug, PartialEq, Eq)]
struct StoredHealth(u32);

impl Component for StoredHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SparseMana(u32);

impl Component for SparseMana {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Debug, PartialEq, Eq)]
struct DetachedProbe(u32);

mod component_storage;
mod despawn_contract;

#[test]
fn archetype_record_is_the_only_dense_table_row_authority() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let record_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/archetype/record.rs")).unwrap();
    let storage_source = std::fs::read_to_string(
        manifest_root.join("src/scene/ecs/storage/component_storage/store.rs"),
    )
    .unwrap();

    assert!(record_source.contains("table: ArchetypeTable"));
    assert!(record_source.contains("self.table.entities()"));
    assert!(!record_source.contains("entities: Vec<EntityId>"));
    assert!(!storage_source.contains("table_components:"));
    assert!(!storage_source.contains("TableComponentStorage"));
}

#[test]
fn entity_registry_reuses_slots_without_accepting_stale_generations() {
    let mut registry = EntityRegistry::default();
    let first = registry
        .spawn(10, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .unwrap();

    assert_eq!(first.index(), 0);
    assert!(registry.contains_internal(first));
    assert_eq!(registry.location_for_stable(10).unwrap().internal, first);

    let despawned = registry.despawn(10).unwrap();
    assert_eq!(despawned.internal, first);
    assert!(!registry.contains_internal(first));

    let second = registry
        .spawn(11, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .unwrap();

    assert_eq!(second.index(), first.index());
    assert_ne!(second.generation(), first.generation());
    assert!(!registry.contains_internal(first));
    assert!(registry.contains_internal(second));
}

#[test]
fn entity_id_reuse_does_not_alias_previous_generation_handle() {
    let mut registry = EntityRegistry::default();
    let stale = registry
        .spawn(100, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .unwrap();

    registry.despawn(100).unwrap();
    let replacement = registry
        .spawn(200, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .unwrap();

    assert_eq!(replacement.index(), stale.index());
    assert_ne!(replacement.generation(), stale.generation());
    assert!(registry.location_for_internal(stale).is_err());
    assert_eq!(
        registry
            .location_for_internal(replacement)
            .unwrap()
            .stable_id,
        200
    );
}

#[test]
fn world_maps_stable_scene_ids_to_internal_generational_entities() {
    let mut world = World::new();
    let cube = world.spawn_node(NodeKind::Cube);
    let internal = world.internal_entity(cube).unwrap();

    assert!(world.contains_entity(cube));
    assert!(world.contains_internal_entity(internal));
    assert_eq!(
        world.internal_entity_location(cube).unwrap().stable_id,
        cube
    );

    world.remove_entity(cube).unwrap();
    assert!(!world.contains_entity(cube));
    assert!(!world.contains_internal_entity(internal));

    let next = world.spawn_node(NodeKind::Cube);
    let next_internal = world.internal_entity(next).unwrap();

    assert_ne!(next, cube);
    assert_eq!(next_internal.index(), internal.index());
    assert_ne!(next_internal.generation(), internal.generation());
}

#[test]
fn despawned_entity_handle_is_rejected_by_world_access() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Old".to_string()), StoredHealth(10)))
        .unwrap();
    let stale_internal = world.internal_entity(entity).unwrap();

    assert_eq!(world.get::<StoredHealth>(entity), Some(&StoredHealth(10)));
    world.remove_entity(entity).unwrap();
    assert!(!world.contains_entity(entity));
    assert_eq!(world.internal_entity(entity), None);
    assert_eq!(world.get::<StoredHealth>(entity), None);
    assert!(world.insert(entity, StoredHealth(20)).is_err());

    let replacement = world
        .spawn((Name("Replacement".to_string()), StoredHealth(30)))
        .unwrap();
    let replacement_internal = world.internal_entity(replacement).unwrap();

    assert_ne!(replacement, entity);
    assert_eq!(replacement_internal.index(), stale_internal.index());
    assert_ne!(
        replacement_internal.generation(),
        stale_internal.generation()
    );
    assert!(!world.contains_internal_entity(stale_internal));
    assert_eq!(world.get::<StoredHealth>(entity), None);
    assert_eq!(
        world.get::<StoredHealth>(replacement),
        Some(&StoredHealth(30))
    );
}

#[test]
fn stable_entity_location_survives_archetype_move_and_invalidates_on_despawn() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Mover".to_string()),)).unwrap();
    let before = world.internal_entity_location(entity).unwrap();

    world.insert(entity, StoredHealth(1)).unwrap();
    let after_insert = world.internal_entity_location(entity).unwrap();

    assert_eq!(after_insert.stable_id, entity);
    assert_eq!(after_insert.internal, before.internal);
    assert_ne!(
        after_insert.location.archetype_id,
        before.location.archetype_id
    );

    world.remove::<StoredHealth>(entity).unwrap().unwrap();
    let after_remove = world.internal_entity_location(entity).unwrap();

    assert_eq!(after_remove.stable_id, entity);
    assert_eq!(after_remove.internal, before.internal);
    assert_eq!(world.get::<StoredHealth>(entity), None);

    world.remove_entity(entity).unwrap();
    assert_eq!(world.internal_entity_location(entity), None);
    assert!(!world.contains_internal_entity(before.internal));
}

#[test]
fn despawn_updates_only_the_swapped_archetype_location() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), StoredHealth(1)))
        .unwrap();
    let removed = world
        .spawn((Name("Removed".to_string()), StoredHealth(2)))
        .unwrap();
    let swapped = world
        .spawn((Name("Swapped".to_string()), StoredHealth(3)))
        .unwrap();
    let removed_location = world.internal_entity_location(removed).unwrap().location;
    let swapped_before = world.internal_entity_location(swapped).unwrap().location;

    assert_eq!(removed_location.archetype_id, swapped_before.archetype_id);
    world.remove_entity(removed).unwrap();

    let swapped_after = world.internal_entity_location(swapped).unwrap().location;
    assert_eq!(swapped_after.archetype_id, removed_location.archetype_id);
    assert_eq!(swapped_after.table_row, removed_location.table_row);
    assert_eq!(world.get::<StoredHealth>(swapped), Some(&StoredHealth(3)));
    assert_eq!(world.get::<StoredHealth>(first), Some(&StoredHealth(1)));
}

#[test]
fn swap_removed_dense_entity_storage_preserves_stable_world_order_across_query_clone_and_serde() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()),)).unwrap();
    let middle = world.spawn((Name("Middle".to_string()),)).unwrap();
    let last = world.spawn((Name("Last".to_string()),)).unwrap();

    world.remove_entity(first).unwrap();

    let query = world.query::<&Name>();
    assert_eq!(
        query
            .iter(&world)
            .map(|name| name.0.clone())
            .collect::<Vec<_>>(),
        vec!["Middle", "Last"]
    );

    let cloned = world.clone();
    let clone_query = cloned.query::<&Name>();
    assert_eq!(
        clone_query
            .iter(&cloned)
            .map(|name| name.0.clone())
            .collect::<Vec<_>>(),
        vec!["Middle", "Last"]
    );

    let restored: World = serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    let restored_query = restored.query::<&Name>();
    assert_eq!(
        restored_query
            .iter(&restored)
            .map(|name| name.0.clone())
            .collect::<Vec<_>>(),
        vec!["Middle", "Last"]
    );
    assert!(!restored.contains_entity(first));
    assert!(restored.contains_entity(middle));
    assert!(restored.contains_entity(last));
}

#[test]
fn clone_and_serde_projection_rebuild_append_one_final_archetype_row_per_entity() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()),)).unwrap();
    let second = world.spawn((Name("Second".to_string()),)).unwrap();
    world
        .set_dynamic_component(
            first,
            "test.persisted_presence",
            serde_json::json!({ "x": 1 }),
        )
        .unwrap();
    world.reset_ecs_frame_performance_diagnostics();

    let cloned = world.clone();
    assert_eq!(
        cloned
            .ecs_frame_performance_diagnostics()
            .archetype_index
            .row_appends,
        2
    );
    assert_eq!(cloned.get::<Name>(first).unwrap().0, "First");
    assert_eq!(cloned.get::<Name>(second).unwrap().0, "Second");
    assert_eq!(
        cloned.dynamic_component(first, "test.persisted_presence"),
        Some(&serde_json::json!({ "x": 1 }))
    );

    let restored: World = serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    assert_eq!(
        restored
            .ecs_frame_performance_diagnostics()
            .archetype_index
            .row_appends,
        2
    );
    assert_eq!(restored.get::<Name>(first).unwrap().0, "First");
    assert_eq!(restored.get::<Name>(second).unwrap().0, "Second");
    assert_eq!(
        restored.dynamic_component(first, "test.persisted_presence"),
        Some(&serde_json::json!({ "x": 1 }))
    );
}

#[test]
fn detached_subtree_restores_table_sparse_hierarchy_and_stable_order_without_world_clone() {
    let mut world = World::empty();
    let before = world.spawn((Name("Before".to_string()),)).unwrap();
    let parent = world
        .spawn((Name("Parent".to_string()), StoredHealth(10), SparseMana(20)))
        .unwrap();
    let child = world
        .spawn((Name("Child".to_string()), StoredHealth(30), SparseMana(40)))
        .unwrap();
    let after = world.spawn((Name("After".to_string()),)).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    world
        .set_dynamic_component(
            parent,
            "test.detached_payload",
            serde_json::json!({ "value": 9 }),
        )
        .unwrap();
    let observer_values = Arc::new(Mutex::new(Vec::new()));
    let observer_values_for_callback = Arc::clone(&observer_values);
    world.observe_entity_event::<DetachedProbe>(child, move |_world, entity, event| {
        observer_values_for_callback
            .lock()
            .unwrap()
            .push((entity, event.0));
    });
    let parent_health_ticks = world
        .component_change_ticks::<StoredHealth>(parent)
        .unwrap();
    let child_sparse_ticks = world.component_change_ticks::<SparseMana>(child).unwrap();
    world.reset_ecs_frame_performance_diagnostics();

    let batch = world.remove_entity_recursive(parent).unwrap();

    let detach_diagnostics = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(detach_diagnostics.commit_count, 1);
    assert_eq!(detach_diagnostics.rejected_preflights, 0);
    assert_eq!(detach_diagnostics.full_world_clone_bytes, 0);
    assert_eq!(detach_diagnostics.node_record_clone_bytes, 0);
    assert_eq!(detach_diagnostics.rollback_bytes, 0);
    assert_eq!(detach_diagnostics.moved_rows, 2);
    assert_eq!(detach_diagnostics.moved_dynamic_components, 1);
    assert_eq!(detach_diagnostics.archetype_publications, 2);
    assert_eq!(detach_diagnostics.generation_advances, 1);
    assert_eq!(detach_diagnostics.ordered_removals, 2);
    assert_eq!(detach_diagnostics.hierarchy_index_lookups, 2);
    assert!(detach_diagnostics.moved_table_components >= 2);
    assert!(detach_diagnostics.moved_sparse_components >= 2);
    assert!(detach_diagnostics.lifecycle_events >= 4);

    assert!(!world.contains_entity(parent));
    assert!(!world.contains_entity(child));
    assert_eq!(
        world
            .query::<&Name>()
            .iter(&world)
            .map(|name| name.0.as_str())
            .collect::<Vec<_>>(),
        vec!["Before", "After"]
    );

    world.restore_detached_entity_batch(batch).unwrap();

    let restored_diagnostics = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(restored_diagnostics.commit_count, 2);
    assert_eq!(restored_diagnostics.moved_rows, 4);
    assert_eq!(restored_diagnostics.moved_dynamic_components, 2);
    assert_eq!(restored_diagnostics.archetype_publications, 4);
    assert_eq!(restored_diagnostics.generation_advances, 2);
    assert_eq!(restored_diagnostics.ordered_removals, 2);

    assert_eq!(world.get::<StoredHealth>(parent), Some(&StoredHealth(10)));
    assert_eq!(world.get::<SparseMana>(parent), Some(&SparseMana(20)));
    assert_eq!(world.get::<StoredHealth>(child), Some(&StoredHealth(30)));
    assert_eq!(world.get::<SparseMana>(child), Some(&SparseMana(40)));
    assert_eq!(
        world.dynamic_component(parent, "test.detached_payload"),
        Some(&serde_json::json!({ "value": 9 }))
    );
    assert_eq!(
        world.component_change_ticks::<StoredHealth>(parent),
        Some(parent_health_ticks)
    );
    assert_eq!(
        world.component_change_ticks::<SparseMana>(child),
        Some(child_sparse_ticks)
    );
    assert_eq!(world.parent_of(child), Some(parent));
    world.trigger_entity_event(child, DetachedProbe(17));
    assert_eq!(*observer_values.lock().unwrap(), vec![(child, 17)]);
    assert_eq!(
        world
            .query::<&Name>()
            .iter(&world)
            .map(|name| name.0.as_str())
            .collect::<Vec<_>>(),
        vec!["Before", "Parent", "Child", "After"]
    );
    assert!(world.contains_entity(before));
    assert!(world.contains_entity(after));
}

#[test]
fn detached_active_camera_uses_indexed_fallback_and_restores_original_owner() {
    let mut world = World::empty();
    let first_camera = world
        .spawn((Name("First Camera".to_string()), CameraComponent::default()))
        .unwrap();
    let active_camera = world
        .spawn((
            Name("Active Camera".to_string()),
            CameraComponent::default(),
        ))
        .unwrap();
    world.set_active_camera(active_camera);

    let batch = world.remove_entity_recursive(active_camera).unwrap();

    assert_eq!(world.active_camera(), first_camera);
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.active_camera(), active_camera);
}

#[test]
fn restoring_non_camera_batch_does_not_overwrite_new_active_camera_selection() {
    let mut world = World::empty();
    let first_camera = world
        .spawn((Name("First Camera".to_string()), CameraComponent::default()))
        .unwrap();
    let second_camera = world
        .spawn((
            Name("Second Camera".to_string()),
            CameraComponent::default(),
        ))
        .unwrap();
    let detached = world.spawn((Name("Detached".to_string()),)).unwrap();
    world.set_active_camera(first_camera);

    let batch = world.remove_entity_recursive(detached).unwrap();
    world.set_active_camera(second_camera);
    world.restore_detached_entity_batch(batch).unwrap();

    assert_eq!(world.active_camera(), second_camera);
}

#[test]
fn detached_subtree_roots_are_deduplicated_and_ancestor_covered_once() {
    let mut world = World::empty();
    let root = world.spawn((Name("Root".to_string()),)).unwrap();
    let child = world.spawn((Name("Child".to_string()),)).unwrap();
    let grandchild = world.spawn((Name("Grandchild".to_string()),)).unwrap();
    world.set_parent_checked(child, Some(root)).unwrap();
    world.set_parent_checked(grandchild, Some(child)).unwrap();
    world.reset_ecs_frame_performance_diagnostics();

    let batch = world
        .remove_entity_subtrees([child, root, child, grandchild])
        .unwrap();

    assert_eq!(
        batch.entity_ids().collect::<Vec<_>>(),
        vec![root, child, grandchild]
    );
    let diagnostics = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(diagnostics.moved_rows, 3);
    assert_eq!(diagnostics.ordered_removals, 3);
    assert_eq!(diagnostics.archetype_publications, 3);
    assert_eq!(diagnostics.generation_advances, 1);

    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(child), Some(root));
    assert_eq!(world.parent_of(grandchild), Some(child));
}

#[test]
fn detached_subtree_uses_hierarchy_postorder_when_child_is_older_than_parent() {
    let mut world = World::empty();
    let older_child = world.spawn((Name("Older Child".to_string()),)).unwrap();
    let newer_parent = world.spawn((Name("Newer Parent".to_string()),)).unwrap();
    world
        .set_parent_checked(older_child, Some(newer_parent))
        .unwrap();

    let batch = world.remove_entity_recursive(newer_parent).unwrap();

    assert_eq!(
        batch.entity_ids().collect::<Vec<_>>(),
        vec![older_child, newer_parent]
    );
    assert!(world.is_empty());
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(older_child), Some(newer_parent));
}

#[test]
fn rejected_detached_batch_returns_ownership_without_mutating_existing_entity() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Detached".to_string()), StoredHealth(7)))
        .unwrap();
    let batch = world.remove_entity_recursive(entity).unwrap();
    world
        .spawn_at(entity, (Name("Conflict".to_string()), StoredHealth(9)))
        .unwrap();
    world.reset_ecs_frame_performance_diagnostics();

    let error = world.restore_detached_entity_batch(batch).unwrap_err();
    assert!(matches!(
        error.error(),
        crate::scene::SceneError::DuplicateEntity { entity: duplicate } if *duplicate == entity
    ));
    let (_, batch) = error.into_parts();
    assert_eq!(world.get::<StoredHealth>(entity), Some(&StoredHealth(9)));
    let rejected = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(rejected.rejected_preflights, 1);
    assert_eq!(rejected.commit_count, 0);
    assert_eq!(rejected.rollback_bytes, 0);
    assert_eq!(rejected.full_world_clone_bytes, 0);
    assert_eq!(rejected.node_record_clone_bytes, 0);

    world.remove_entity(entity).unwrap();
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.get::<StoredHealth>(entity), Some(&StoredHealth(7)));
}

#[test]
fn explicit_empty_spawn_updates_only_the_empty_archetype() {
    let mut world = World::empty();
    assert!(world.spawn_empty_at(40).unwrap());
    assert!(world.spawn_empty_at(80).unwrap());

    let first = world.internal_entity_location(40).unwrap().location;
    let second = world.internal_entity_location(80).unwrap().location;
    assert_eq!(first.archetype_id, ArchetypeId::EMPTY);
    assert_eq!(second.archetype_id, ArchetypeId::EMPTY);
    assert_eq!(first.table_row, 0);
    assert_eq!(second.table_row, 1);
}

#[test]
fn empty_entity_archetype_placement_rejects_a_preallocated_but_unowned_locator() {
    let source = include_str!("../world/identity.rs");
    let location_lookup = source
        .split("fn archetype_location_for_entity")
        .nth(1)
        .and_then(|text| text.split("fn update_entity_archetype_row").next())
        .expect("read entity archetype location lookup");
    let location_lookup_compact = location_lookup.split_whitespace().collect::<String>();

    assert!(
        location_lookup_compact.contains("self.archetype_index.entities(location.archetype_id)")
            && location_lookup_compact.contains(".and_then(|entities|entities.get(location.table_row))")
            && location_lookup_compact.contains("(located_entity==Some(entity)).then_some"),
        "a registry preallocation must not be treated as an existing archetype row before the index owns that entity"
    );
}

#[test]
fn explicit_empty_spawn_does_not_rebuild_all_archetypes() {
    let source = include_str!("../world/typed_api/bundle_entry.rs");
    let spawn_empty = source
        .split("pub(crate) fn spawn_empty_at(")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn spawn_at").next())
        .expect("read World::spawn_empty_at body");

    assert!(
        spawn_empty.contains("self.register_stable_entity(entity)?;")
            && !spawn_empty.contains("place_empty_entity_in_archetype")
            && !spawn_empty.contains("self.refresh_stable_entity_locations();"),
        "stable entity registration must publish the only empty-archetype row without a second assignment facade"
    );
}

#[test]
fn world_contains_entity_uses_entity_registry_membership() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("query.rs"),
    )
    .unwrap();
    let contains_entity = source
        .split("pub fn contains_entity")
        .nth(1)
        .and_then(|text| text.split("pub fn camera_count").next())
        .expect("read contains_entity body");

    assert!(
        contains_entity.contains("self.entity_registry.contains_stable(entity)")
            && !contains_entity.contains("self.entities.contains(&entity)"),
        "World::contains_entity must use the stable-id entity registry instead of scanning the world entity list"
    );
}

#[test]
fn stable_entity_registration_appends_the_empty_archetype_row_before_publishing_location() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    )
    .unwrap();
    let registration = source
        .split("pub(super) fn register_stable_entity")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn unregister_stable_entity").next())
        .expect("read stable entity registration body");
    let registration_compact = registration.split_whitespace().collect::<String>();

    assert!(
        registration_compact.contains(
            "letinternal=self.entity_registry.spawn(entity,EntityLocation::new(ArchetypeId::EMPTY,usize::MAX))?;"
        )
            && registration.contains("let row = self.append_empty_archetype_row(entity);")
            && registration_compact.contains(
                "self.entity_registry.set_location(entity,EntityLocation::new(ArchetypeId::EMPTY,row))"
            )
            && registration.contains("self.stable_query_order.register(entity, internal);")
            && registration_compact.contains(
                "self.stable_query_order.move_to(entity,EntityLocation::new(ArchetypeId::EMPTY,row));"
            )
            && registration_compact.contains("Ok(internal)")
            && !registration.contains(".iter()")
            && !registration.contains(".position(|candidate| *candidate == entity)")
            && !registration.contains("unwrap_or(self.entities.len())")
            && !registration.contains(".map_err("),
        "stable entity registration must materialize the empty table row before publishing the only dense locator"
    );
    assert!(
        !source.contains("fn entity_registry_error_to_string"),
        "stable entity registration must not keep a helper solely for map_err conversion"
    );
}

#[test]
fn dynamic_component_membership_uses_explicit_row_transitions_without_facades() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    )
    .unwrap();
    let insert = source
        .split("pub(super) fn insert_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn remove_dynamic_component_presence")
                .next()
        })
        .expect("read dynamic component presence insert body");
    let remove = source
        .split("pub(super) fn remove_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn rebuild_typed_component_presence")
                .next()
        })
        .expect("read dynamic component presence removal body");

    assert!(
        insert.contains("self.begin_component_row(entity)")
            && insert.contains("self.commit_component_row(entity, row, false)")
            && remove.contains(".with_component_removed(component_id, StorageType::SparseSet)")
            && remove.contains("self.transition_entity_archetype_row(")
            && !source.contains("add_component_to_entity_archetype")
            && !source.contains("remove_component_from_entity_archetype"),
        "dynamic sparse membership must publish explicit complete-row transitions without signature-assignment facades"
    );
}

#[test]
fn entity_registry_location_for_stable_uses_direct_internal_lookup() {
    let source = include_str!("../ecs/entity/registry.rs");
    let location_for_stable = source
        .split("pub fn location_for_stable(&self, stable_id: EntityId)")
        .nth(1)
        .and_then(|text| text.split("pub fn location_for_internal").next())
        .expect("read EntityRegistry::location_for_stable body");

    assert!(
        location_for_stable
            .contains("let Some(internal) = self.internal_for_stable(stable_id) else")
            && location_for_stable.contains("return None;")
            && location_for_stable.contains("self.location_for_internal(internal).ok()")
            && !location_for_stable.contains(".and_then(|internal|"),
        "stable location lookup must branch directly from stable id to internal generational location"
    );
}

#[test]
fn entity_registry_error_paths_use_direct_lookup_branches() {
    let source = include_str!("../ecs/entity/registry.rs");
    let despawn = source
        .split("pub fn despawn(&mut self, stable_id: EntityId)")
        .nth(1)
        .and_then(|text| text.split("pub fn clear(&mut self)").next())
        .expect("read EntityRegistry despawn body");
    let set_location = source
        .split("pub fn set_location(")
        .nth(1)
        .and_then(|text| text.split("pub fn contains_internal").next())
        .expect("read EntityRegistry set_location body");
    let location_for_internal = source
        .split("pub fn location_for_internal(")
        .nth(1)
        .and_then(|text| text.split("pub fn len(&self)").next())
        .expect("read EntityRegistry location_for_internal body");

    assert!(
        despawn.contains("let Some(internal) = self.stable_to_internal.remove(&stable_id) else")
    );
    assert!(despawn.contains("return Err(EntityRegistryError::MissingStableId(stable_id));"));
    assert!(despawn.contains("let Some(slot) = self.slots.get_mut(internal.index() as usize) else"));
    assert!(set_location.contains("let Some(internal) = self.internal_for_stable(stable_id) else"));
    assert!(set_location.contains("return Err(EntityRegistryError::MissingStableId(stable_id));"));
    assert!(set_location
        .contains("let Some(slot) = self.slots.get_mut(internal.index() as usize) else"));
    assert!(location_for_internal
        .contains("let Some(slot) = self.slots.get(internal.index() as usize) else"));
    assert!(location_for_internal.contains("let Some(stable_id) = slot.stable_id else"));
    assert!(location_for_internal.contains("let Some(location) = slot.location else"));
    assert!(!source.contains(".ok_or("));
    assert!(!source.contains(".ok_or_else("));
}

#[test]
fn entity_registry_despawn_location_take_uses_direct_default_branch() {
    let source = include_str!("../ecs/entity/registry.rs");
    let despawn = source
        .split("pub fn despawn(&mut self, stable_id: EntityId)")
        .nth(1)
        .and_then(|text| text.split("pub fn clear(&mut self)").next())
        .expect("read EntityRegistry despawn body");

    assert!(despawn.contains("let location = match slot.location.take()"));
    assert!(despawn.contains("Some(location) => location"));
    assert!(despawn.contains("None => EntityLocation::default()"));
    assert!(!despawn.contains(".unwrap_or_default()"));
}

#[test]
fn entity_registry_generation_wrap_uses_direct_checked_branch() {
    let source = include_str!("../ecs/entity/slot.rs");
    let next_generation = source
        .split("fn next_generation(generation: u32) -> u32")
        .nth(1)
        .expect("read EntityRegistry generation helper body");

    assert!(next_generation.contains("match generation.checked_add(1)"));
    assert!(next_generation.contains("Some(next_generation) => next_generation"));
    assert!(next_generation.contains("None => FIRST_GENERATION"));
    assert!(!next_generation.contains(".unwrap_or(FIRST_GENERATION)"));
}

#[test]
fn internal_identity_map_is_rebuilt_after_scene_roundtrip_without_serializing_runtime_slots() {
    let mut world = World::new();
    let imported = world.spawn_node(NodeKind::Mesh);

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_scene_identity_roundtrip_{unique}.json"));
    world.save_project_to_path(&path).unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert!(!saved.contains("entity_registry"));
    assert_eq!(
        loaded.internal_entity_location(imported).unwrap().stable_id,
        imported
    );
}
