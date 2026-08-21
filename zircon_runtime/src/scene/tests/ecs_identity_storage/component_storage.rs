use super::*;
use crate::scene::{ChangeTick, ComponentId, ComponentStorage, ComponentTicks, InternalEntity};

#[test]
fn component_storage_rejects_dense_values_and_keeps_sparse_locations_addressable() {
    let table_component = ComponentId::new(1);
    let sparse_component = ComponentId::new(2);
    let first = InternalEntity::new(0, 1);
    let mut storage = ComponentStorage::default();

    let error = storage
        .insert(
            table_component,
            StorageType::Table,
            first,
            TestComponent("first"),
        )
        .expect_err("dense components must be owned by an archetype table");
    assert!(matches!(
        error,
        crate::scene::ecs::StorageError::TableOwnedByArchetype { component_id }
            if component_id == table_component
    ));
    storage
        .insert(
            sparse_component,
            StorageType::SparseSet,
            first,
            TestComponent("sparse"),
        )
        .unwrap();

    let sparse_location = storage
        .location(sparse_component, first)
        .expect("sparse insert must publish a location");
    assert_eq!(sparse_location.storage_type, StorageType::SparseSet);
    assert_eq!(sparse_location.entity, first);
    assert_eq!(sparse_location.table_row, None);
    assert_eq!(sparse_location.table_archetype, None);
    assert_eq!(sparse_location.table_column_slot, None);
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(sparse_location),
        Some((
            &TestComponent("sparse"),
            ComponentTicks::new(ChangeTick::INITIAL)
        ))
    );

    let sparse_removed = storage
        .remove::<TestComponent>(sparse_component, first)
        .unwrap()
        .unwrap();

    assert_eq!(sparse_removed.value, TestComponent("sparse"));
    assert!(!storage.contains(sparse_component, first));
    assert_eq!(
        storage.get_with_ticks_at_location::<TestComponent>(sparse_location),
        None
    );
}

#[test]
fn component_storage_sparse_iteration_never_exposes_dense_table_values() {
    let table_component = ComponentId::new(31);
    let sparse_component = ComponentId::new(32);
    let first = InternalEntity::new(0, 1);
    let second = InternalEntity::new(1, 1);
    let mut storage = ComponentStorage::default();

    assert!(
        storage
            .insert(
                table_component,
                StorageType::Table,
                first,
                TestComponent("table"),
            )
            .is_err()
    );
    storage
        .insert(
            sparse_component,
            StorageType::SparseSet,
            second,
            TestComponent("sparse"),
        )
        .expect("sparse component should insert");

    let mut sparse_entities = Vec::new();
    storage.for_each_sparse_entity(sparse_component, |entity| sparse_entities.push(entity));
    assert_eq!(sparse_entities, vec![second]);
}

#[test]
fn sparse_component_storage_removal_keeps_the_swapped_entity_addressable() {
    let component = ComponentId::new(8);
    let first = InternalEntity::new(0, 1);
    let second = InternalEntity::new(1, 1);
    let second_added = ChangeTick::new(41);
    let second_changed = ChangeTick::new(73);
    let mut storage = ComponentStorage::default();

    storage
        .insert(
            component,
            StorageType::SparseSet,
            first,
            TestComponent("first"),
        )
        .unwrap();
    storage
        .insert_at_tick(
            component,
            StorageType::SparseSet,
            second,
            TestComponent("second"),
            second_added,
        )
        .unwrap();
    storage.mark_changed(component, second, second_changed);

    let removed = storage
        .remove::<TestComponent>(component, first)
        .unwrap()
        .unwrap();

    assert_eq!(removed.value, TestComponent("first"));
    assert!(!storage.contains(component, first));
    assert_eq!(
        storage.get::<TestComponent>(component, second),
        Some(&TestComponent("second"))
    );
    let mut expected_ticks = ComponentTicks::new(second_added);
    expected_ticks.set_changed(second_changed);
    assert_eq!(storage.ticks(component, second), Some(expected_ticks));
}

#[test]
fn sparse_component_locator_rejects_a_reused_internal_slot_generation() {
    let source = include_str!("../../ecs/storage/component_storage/sparse.rs");
    assert!(source.contains("sparse_rows: Vec<Option<SparseRowLocation>>"));
    assert!(source.contains("generation: u32"));
    assert!(source.contains("dense_row: usize"));
    assert!(!source.contains("HashMap<InternalEntity, usize>"));

    let component = ComponentId::new(81);
    let stale = InternalEntity::new(7, 3);
    let replacement = InternalEntity::new(7, 4);
    let mut storage = ComponentStorage::default();

    storage
        .insert(
            component,
            StorageType::SparseSet,
            stale,
            TestComponent("stale"),
        )
        .unwrap();

    assert_eq!(
        storage.get::<TestComponent>(component, stale),
        Some(&TestComponent("stale"))
    );
    assert_eq!(storage.get::<TestComponent>(component, replacement), None);
    assert!(!storage.contains(component, replacement));

    let removed = storage
        .remove::<TestComponent>(component, stale)
        .unwrap()
        .unwrap();
    assert_eq!(removed.value, TestComponent("stale"));
    storage
        .insert(
            component,
            StorageType::SparseSet,
            replacement,
            TestComponent("replacement"),
        )
        .unwrap();
    assert_eq!(storage.get::<TestComponent>(component, stale), None);
    assert_eq!(
        storage.get::<TestComponent>(component, replacement),
        Some(&TestComponent("replacement"))
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
            StorageType::SparseSet,
            entity,
            TestComponent("typed"),
        )
        .unwrap();

    assert!(
        storage
            .insert(
                component,
                StorageType::Table,
                entity,
                TestComponent("moved")
            )
            .unwrap_err()
            .to_string()
            .contains("owned by its ArchetypeTable")
    );
    assert!(
        storage
            .insert(component, StorageType::SparseSet, entity, "wrong-type")
            .unwrap_err()
            .to_string()
            .contains("different Rust type")
    );
    assert_eq!(
        storage.get::<TestComponent>(component, entity),
        Some(&TestComponent("typed"))
    );
}
