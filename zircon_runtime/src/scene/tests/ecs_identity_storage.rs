use crate::scene::ecs::{
    ArchetypeId, ChangeTick, ComponentId, ComponentStorage, ComponentStorageLocation,
    ComponentTicks, EntityLocation, EntityRegistry, InternalEntity, StorageType,
};
use crate::scene::{NodeKind, World};

#[derive(Debug, PartialEq, Eq)]
struct TestComponent(&'static str);

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
