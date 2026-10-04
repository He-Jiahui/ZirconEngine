use super::super::slot::FIRST_GENERATION;
use super::*;

#[test]
fn generation_exhaustion_retires_the_slot_instead_of_reusing_a_stale_handle() {
    let mut registry = EntityRegistry::default();
    let original = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first entity must allocate a slot");
    let exhausted = InternalEntity::new(original.index(), u32::MAX);

    registry.slots[original.index() as usize].generation = u32::MAX;
    let replaced = registry.stable_to_internal.insert(1, exhausted);
    assert_eq!(replaced, Some(original));

    let despawned = registry
        .despawn(1)
        .expect("an entity at the generation limit must still despawn");
    assert_eq!(despawned.internal, exhausted);
    assert!(!registry.free_slots.contains(&original.index()));

    let replacement = registry
        .spawn(2, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("a retired slot must not prevent allocation of another slot");
    assert_ne!(replacement.index(), original.index());
    assert_eq!(replacement.generation(), FIRST_GENERATION);
    assert!(!registry.contains_internal(exhausted));
}

#[test]
fn invalid_internal_handle_does_not_remove_the_stable_entity_mapping() {
    let mut registry = EntityRegistry::default();
    let internal = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first entity must allocate a slot");

    registry.slots[internal.index() as usize].generation += 1;

    assert_eq!(
        registry.despawn(1),
        Err(EntityRegistryError::InvalidInternalEntity)
    );
    assert_eq!(registry.internal_for_stable(1), Some(internal));
}

#[test]
fn slot_capacity_exhaustion_returns_a_typed_error_without_mutation() {
    let mut registry = EntityRegistry::with_max_slots(1);
    let first = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first slot must be admitted");

    assert_eq!(
        registry.spawn(2, EntityLocation::new(ArchetypeId::EMPTY, 1)),
        Err(EntityRegistryError::SlotCapacityExhausted { max_slots: 1 })
    );
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.internal_for_stable(1), Some(first));
    assert!(!registry.contains_stable(2));
}

#[test]
fn prevalidated_duplicate_rejection_returns_a_typed_error_without_mutation() {
    let mut registry = EntityRegistry::with_max_slots(2);
    let original = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first entity must allocate a slot");

    assert_eq!(
        registry.spawn_prevalidated(1, EntityLocation::new(ArchetypeId::EMPTY, 1)),
        Err(EntityRegistryError::DuplicateStableId(1))
    );
    assert_eq!(registry.internal_for_stable(1), Some(original));
    assert_eq!(registry.len(), 1);
}

#[test]
fn rebuild_capacity_rejection_preserves_the_existing_registry() {
    let mut registry = EntityRegistry::with_max_slots(1);
    let original = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first slot must be admitted");

    assert_eq!(
        registry.rebuild_from_stable_ids([2, 3]),
        Err(EntityRegistryError::SlotCapacityExhausted { max_slots: 1 })
    );
    assert_eq!(registry.internal_for_stable(1), Some(original));
    assert_eq!(registry.len(), 1);
}

#[test]
fn rebuild_duplicate_rejection_preserves_the_existing_registry() {
    let mut registry = EntityRegistry::with_max_slots(2);
    let original = registry
        .spawn(1, EntityLocation::new(ArchetypeId::EMPTY, 0))
        .expect("the first slot must be admitted");

    assert_eq!(
        registry.rebuild_from_stable_ids([2, 2]),
        Err(EntityRegistryError::DuplicateStableId(2))
    );
    assert_eq!(registry.internal_for_stable(1), Some(original));
    assert_eq!(registry.len(), 1);
}
