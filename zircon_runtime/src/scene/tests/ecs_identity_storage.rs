use crate::scene::components::Name;
use crate::scene::ecs::{
    ArchetypeId, ChangeTick, Component, ComponentId, ComponentStorage, ComponentStorageLocation,
    ComponentTicks, EntityLocation, EntityRegistry, InternalEntity, StorageType,
};
use crate::scene::{NodeKind, World};

#[derive(Debug, PartialEq, Eq)]
struct TestComponent(&'static str);

#[derive(Debug, PartialEq, Eq)]
struct StoredHealth(u32);

impl Component for StoredHealth {}

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

    assert!(world.remove_entity(cube));
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
    assert!(world.remove_entity(entity));
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

    assert!(world.remove_entity(entity));
    assert_eq!(world.internal_entity_location(entity), None);
    assert!(!world.contains_internal_entity(before.internal));
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
fn stable_entity_registration_uses_append_row_without_entity_scan() {
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
        registration.contains("let row = self.entities.len();")
            && registration_compact.contains(
                "letinternal=matchself.entity_registry.spawn(entity,EntityLocation::new(ArchetypeId::EMPTY,row))"
            )
            && registration_compact.contains("Ok(internal)=>internal")
            && registration_compact.contains("Err(error)=>returnErr(error.to_string())")
            && registration_compact.contains("Ok(internal)")
            && !registration.contains(".iter()")
            && !registration.contains(".position(|candidate| *candidate == entity)")
            && !registration.contains("unwrap_or(self.entities.len())")
            && !registration.contains(".map_err("),
        "stable entity registration must use the append row and direct registry-spawn result branches"
    );
    assert!(
        !source.contains("fn entity_registry_error_to_string"),
        "stable entity registration must not keep a helper solely for map_err conversion"
    );
}

#[test]
fn entity_archetype_refresh_uses_direct_previous_archetype_branch() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    )
    .unwrap();
    let refresh = source
        .split("pub(super) fn refresh_entity_archetype")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn rebuild_archetype_index").next())
        .expect("read entity archetype refresh body");

    assert!(
        refresh.contains("let previous = match self.entity_registry.location_for_stable(entity)")
            && refresh.contains("Some(location) => Some(location.location.archetype_id)")
            && refresh.contains("None => None")
            && refresh.contains("self.assign_entity_archetype(entity, previous)")
            && !refresh.contains(".map(|location| location.location.archetype_id)"),
        "entity archetype refresh must branch directly on the previous stable location"
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
        location_for_stable.contains("let Some(internal) = self.internal_for_stable(stable_id) else")
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

#[test]
fn component_storage_supports_table_swap_remove_and_sparse_remove() {
    let table_component = ComponentId::new(1);
    let sparse_component = ComponentId::new(2);
    let first = InternalEntity::new(0, 1);
    let second = InternalEntity::new(1, 1);
    let third = InternalEntity::new(2, 1);
    let mut storage = ComponentStorage::default();

    storage
        .insert(
            table_component,
            StorageType::Table,
            first,
            TestComponent("first"),
        )
        .unwrap();
    storage
        .insert(
            table_component,
            StorageType::Table,
            second,
            TestComponent("second"),
        )
        .unwrap();
    storage
        .insert(
            table_component,
            StorageType::Table,
            third,
            TestComponent("third"),
        )
        .unwrap();
    storage
        .insert(
            sparse_component,
            StorageType::SparseSet,
            first,
            TestComponent("sparse"),
        )
        .unwrap();

    let second_location = ComponentStorageLocation {
        component_id: table_component,
        storage_type: StorageType::Table,
        entity: second,
        table_row: Some(1),
    };
    let sparse_location = ComponentStorageLocation {
        component_id: sparse_component,
        storage_type: StorageType::SparseSet,
        entity: first,
        table_row: None,
    };

    assert_eq!(
        storage.location(table_component, second),
        Some(second_location)
    );
    assert_eq!(
        storage.get_table_row::<TestComponent>(table_component, 2),
        Some((
            third,
            &TestComponent("third"),
            ComponentTicks::new(ChangeTick::INITIAL)
        ))
    );
    assert_eq!(
        storage.location(sparse_component, first),
        Some(sparse_location)
    );
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(second_location),
        Some((
            &TestComponent("second"),
            ComponentTicks::new(ChangeTick::INITIAL)
        ))
    );
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(sparse_location),
        Some((
            &TestComponent("sparse"),
            ComponentTicks::new(ChangeTick::INITIAL)
        ))
    );

    let removed = storage
        .remove::<TestComponent>(table_component, second)
        .unwrap()
        .unwrap();

    assert_eq!(removed.value, TestComponent("second"));
    assert_eq!(removed.swapped_entity, Some(third));
    assert!(!storage.contains(table_component, second));
    assert_eq!(
        storage.get::<TestComponent>(table_component, third),
        Some(&TestComponent("third"))
    );
    assert_eq!(
        storage.location(table_component, third),
        Some(ComponentStorageLocation {
            component_id: table_component,
            storage_type: StorageType::Table,
            entity: third,
            table_row: Some(1),
        })
    );
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(second_location),
        None
    );

    let sparse_removed = storage
        .remove::<TestComponent>(sparse_component, first)
        .unwrap()
        .unwrap();

    assert_eq!(sparse_removed.value, TestComponent("sparse"));
    assert_eq!(sparse_removed.swapped_entity, None);
    assert!(!storage.contains(sparse_component, first));
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(sparse_location),
        None
    );
}

#[test]
fn component_storage_rejects_storage_and_type_mismatches_without_mutating_value() {
    let component = ComponentId::new(7);
    let entity = InternalEntity::new(0, 1);
    let mut storage = ComponentStorage::default();

    storage
        .insert(
            component,
            StorageType::Table,
            entity,
            TestComponent("typed"),
        )
        .unwrap();

    assert!(storage
        .insert(
            component,
            StorageType::SparseSet,
            entity,
            TestComponent("moved")
        )
        .unwrap_err()
        .to_string()
        .contains("already registered as Table"));
    assert!(storage
        .insert(component, StorageType::Table, entity, "wrong-type")
        .unwrap_err()
        .to_string()
        .contains("different Rust type"));
    assert_eq!(
        storage.get::<TestComponent>(component, entity),
        Some(&TestComponent("typed"))
    );
}
