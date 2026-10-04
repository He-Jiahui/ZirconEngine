use super::*;

#[derive(Debug, PartialEq, Eq)]
struct SparseValue(u32);

#[test]
fn sparse_rows_rekey_at_the_target_tick() {
    let component_id = ComponentId::new(1);
    let source_entity = InternalEntity::new(7, 1);
    let target_entity = InternalEntity::new(29, 3);
    let source_tick = ChangeTick::new(11);
    let target_tick = ChangeTick::new(37);
    let mut source = ComponentStorage::default();
    source
        .insert_at_tick(
            component_id,
            StorageType::SparseSet,
            source_entity,
            SparseValue(9),
            source_tick,
        )
        .expect("sparse source row should insert");

    let mut rows = source.extract_entity_rows(source_entity, &[component_id]);
    assert!(!source.contains(component_id, source_entity));
    let row = rows.pop().expect("sparse row should transfer");
    assert_eq!(row.source_ticks(), ComponentTicks::new(source_tick));

    let mut target = ComponentStorage::default();
    target
        .insert_transferred_row(component_id, target_entity, row, target_tick)
        .expect("validated sparse row transfer should succeed");
    assert_eq!(
        target.get::<SparseValue>(component_id, target_entity),
        Some(&SparseValue(9))
    );
    assert_eq!(
        target.ticks(component_id, target_entity),
        Some(ComponentTicks::new(target_tick))
    );
}

#[test]
fn dense_value_insertion_is_rejected_by_the_sparse_owner() {
    let component_id = ComponentId::new(3);
    let error = ComponentStorage::default()
        .insert(
            component_id,
            StorageType::Table,
            InternalEntity::new(1, 0),
            7_u32,
        )
        .expect_err("dense values must be owned by ArchetypeTable");
    assert_eq!(error, StorageError::TableOwnedByArchetype { component_id });
}

#[test]
fn sparse_locator_diagnostics_aggregate_all_sparse_component_owners() {
    let first_component = ComponentId::new(41);
    let second_component = ComponentId::new(42);
    let first_entity = InternalEntity::new(1_000_000_000, 1);
    let second_entity = InternalEntity::new(2_000_000_000, 2);
    let mut storage = ComponentStorage::default();

    storage
        .insert(
            first_component,
            StorageType::SparseSet,
            first_entity,
            SparseValue(1),
        )
        .expect("first sparse component should insert");
    storage
        .insert(
            second_component,
            StorageType::SparseSet,
            second_entity,
            SparseValue(2),
        )
        .expect("second sparse component should insert");

    let populated = storage.sparse_locator_diagnostics();
    assert_eq!(populated.sparse_component_storage_count, 2);
    assert_eq!(populated.locator_entry_count, 2);
    assert_eq!(populated.locator_page_count, 2);
    assert!(populated.locator_allocated_bytes > 0);
    assert!(
        populated.locator_allocated_bytes <= 2 * 16 * 1024,
        "two high-index sparse locators must retain a bounded structural footprint"
    );

    storage
        .remove::<SparseValue>(first_component, first_entity)
        .expect("first sparse component should remove")
        .expect("first sparse value should exist");
    storage
        .remove::<SparseValue>(second_component, second_entity)
        .expect("second sparse component should remove")
        .expect("second sparse value should exist");

    let empty = storage.sparse_locator_diagnostics();
    assert_eq!(empty.sparse_component_storage_count, 2);
    assert_eq!(empty.locator_entry_count, 0);
    assert_eq!(empty.locator_page_count, 0);
    assert_eq!(empty.locator_allocated_bytes, 0);
}
