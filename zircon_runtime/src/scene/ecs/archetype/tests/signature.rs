use super::*;

#[test]
fn component_membership_updates_only_the_known_storage_partition() {
    let signature = ArchetypeSignature::new(vec![ComponentId::new(4)], vec![ComponentId::new(2)]);

    let updated = signature
        .with_component_added(ComponentId::new(3), StorageType::Table)
        .with_component_removed(ComponentId::new(2), StorageType::SparseSet);

    assert_eq!(
        updated.table_components(),
        &[ComponentId::new(3), ComponentId::new(4)]
    );
    assert!(updated.sparse_set_components().is_empty());
    assert_eq!(signature.table_components(), &[ComponentId::new(4)]);
    assert_eq!(signature.sparse_set_components(), &[ComponentId::new(2)]);
}

#[test]
fn component_membership_updates_are_idempotent() {
    let signature = ArchetypeSignature::empty()
        .with_component_added(ComponentId::new(7), StorageType::SparseSet)
        .with_component_added(ComponentId::new(7), StorageType::SparseSet)
        .with_component_removed(ComponentId::new(8), StorageType::SparseSet);

    assert_eq!(signature.sparse_set_components(), &[ComponentId::new(7)]);
}

#[test]
fn ordered_component_ids_merge_table_and_sparse_partitions() {
    let signature = ArchetypeSignature::new(
        vec![ComponentId::new(2), ComponentId::new(6)],
        vec![ComponentId::new(1), ComponentId::new(4)],
    );

    assert_eq!(
        signature.ordered_component_ids(),
        vec![
            ComponentId::new(1),
            ComponentId::new(2),
            ComponentId::new(4),
            ComponentId::new(6),
        ]
    );
}
