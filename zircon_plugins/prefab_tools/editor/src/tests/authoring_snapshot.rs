use crate::{
    prefab_source_snapshot, prefab_source_snapshot_expected, validate_prefab_instance,
    PrefabAuthoringDiagnosticKind, PrefabInstanceAuthoringSnapshot, PrefabSourceAuthoringSnapshot,
    PrefabSourceComponentConflict, PrefabSourceComponentDescriptor, PrefabSourceComponentLookup,
    PrefabSourceDiagnosticKind, PrefabSourceEntityLookup, PrefabSourceEntityParent,
    PrefabSourceReadSnapshot, PrefabSourceSnapshotError, PrefabSourceState,
};
use zircon_runtime::asset::{
    AssetId, AssetKind, AssetReference, AssetRegistryEntry, AssetRegistryIndex, AssetUri,
    PrefabAsset, PrefabInstanceAsset, PrefabPropertyOverrideAsset, SceneAsset,
    SceneComponentAssetRecord, SceneEntityAsset, TransformAsset,
};
use zircon_runtime::core::resource::{
    PrefabMarker, ResourceActiveSnapshotError, ResourceHandle, ResourceManager, ResourceRecord,
};

#[test]
fn prefab_authoring_snapshot_preserves_serialized_rows_and_all_conflict_locations() {
    let instance = instance(vec![
        override_value("船体/长名称组件", "Transform.translation.z", 1),
        override_value("root", "Transform.translation.a", 2),
        override_value("船体/长名称组件", "Transform.translation.z", 3),
    ]);

    let snapshot = PrefabInstanceAuthoringSnapshot::new(&instance, PrefabSourceState::Unchecked);

    assert!(std::ptr::eq(snapshot.instance, &instance));
    assert_eq!(snapshot.override_rows.len(), 3);
    for (source_index, row) in snapshot.override_rows.iter().enumerate() {
        assert_eq!(row.source_index, source_index);
        assert!(std::ptr::eq(
            row.override_value,
            &instance.overrides[source_index]
        ));
    }
    assert_eq!(snapshot.diagnostics.len(), 1);
    assert_eq!(
        snapshot.diagnostics[0].kind,
        PrefabAuthoringDiagnosticKind::DuplicateOverridePath
    );
    assert_eq!(snapshot.diagnostics[0].source_indices, vec![0, 2]);
    assert!(snapshot.diagnostics[0].message.contains("船体/长名称组件"));
}

#[test]
fn prefab_authoring_snapshot_distinguishes_unchecked_available_and_missing_sources() {
    let instance = instance(Vec::new());
    for source_state in [PrefabSourceState::Unchecked, PrefabSourceState::Available] {
        let snapshot = PrefabInstanceAuthoringSnapshot::new(&instance, source_state);
        assert_eq!(snapshot.source_state, source_state);
        assert_eq!(snapshot.instance.prefab, instance.prefab);
        assert!(snapshot.override_rows.is_empty());
        assert!(snapshot.diagnostics.is_empty());
    }

    let missing = PrefabInstanceAuthoringSnapshot::new(&instance, PrefabSourceState::Missing);
    assert_eq!(missing.source_state, PrefabSourceState::Missing);
    assert_eq!(missing.diagnostics.len(), 1);
    assert_eq!(
        missing.diagnostics[0].kind,
        PrefabAuthoringDiagnosticKind::MissingSource
    );
    assert!(missing.diagnostics[0].source_indices.is_empty());
    assert!(missing.diagnostics[0]
        .message
        .contains("res://prefabs/ship.prefab.toml"));
}

#[test]
fn prefab_authoring_diagnostics_locate_each_invalid_row_and_keep_legacy_messages() {
    let instance = instance(vec![
        override_value("root", "Transform.translation.x", 1),
        override_value(" ", "", 2),
        override_value(" ", "", 3),
        override_value("root", "Transform.translation.x", 4),
    ]);

    let snapshot = PrefabInstanceAuthoringSnapshot::new(&instance, PrefabSourceState::Missing);
    let locations = |kind| {
        snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == kind)
            .map(|diagnostic| diagnostic.source_indices.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        locations(PrefabAuthoringDiagnosticKind::EmptyEntityPath),
        vec![vec![1], vec![2]]
    );
    assert_eq!(
        locations(PrefabAuthoringDiagnosticKind::EmptyPropertyPath),
        vec![vec![1], vec![2]]
    );
    assert_eq!(
        locations(PrefabAuthoringDiagnosticKind::DuplicateOverridePath),
        vec![vec![1, 2], vec![0, 3]]
    );
    assert_eq!(
        locations(PrefabAuthoringDiagnosticKind::MissingSource),
        vec![Vec::<usize>::new()]
    );
    assert_eq!(
        validate_prefab_instance(&instance, false),
        vec![
            "duplicate prefab override ` ` / `` is ambiguous".to_string(),
            "duplicate prefab override `root` / `Transform.translation.x` is ambiguous".to_string(),
            "prefab instance source `res://prefabs/ship.prefab.toml` is not available".to_string(),
            "prefab override entity path must not be empty".to_string(),
            "prefab override property path must not be empty".to_string(),
        ]
    );
}

#[test]
fn prefab_instance_source_snapshot_checks_relocated_source_and_borrows_all_opaque_overrides() {
    let instance = instance(vec![
        override_value("船体/长名称组件", "Transform.translation.z", 1),
        override_value("船体/长名称组件", "Transform.translation.z", 3),
        PrefabPropertyOverrideAsset {
            entity_path: "opaque/路径".to_string(),
            property_path: "test.Component.未解码属性".to_string(),
            value: serde_json::json!({"unknown": [u64::MAX, {"保留": true}]}),
        },
    ]);
    let before = instance.clone();
    let current = AssetUri::parse("res://prefabs/迁移/ship.prefab.toml").unwrap();
    let registry = source_registry(&instance.prefab, &current, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, current.clone()),
            source_asset(&current, "ready empty source"),
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();

    let snapshot = read
        .instance_authoring_snapshot(&instance, &registry, &resources)
        .unwrap();

    assert!(std::ptr::eq(snapshot.instance, &instance));
    assert_eq!(snapshot.source_state, PrefabSourceState::Available);
    assert_eq!(snapshot.override_rows.len(), 3);
    for (index, row) in snapshot.override_rows.iter().enumerate() {
        assert_eq!(row.source_index, index);
        assert!(std::ptr::eq(row.override_value, &instance.overrides[index]));
    }
    assert_eq!(snapshot.diagnostics.len(), 1);
    assert_eq!(
        snapshot.diagnostics[0].kind,
        PrefabAuthoringDiagnosticKind::DuplicateOverridePath
    );
    assert_eq!(snapshot.diagnostics[0].source_indices, vec![0, 1]);
    assert!(read.authoring_snapshot().entity_rows.is_empty());
    assert_eq!(read.source_snapshot().uri, current);
    assert_eq!(instance, before);
}

#[test]
fn prefab_instance_source_snapshot_rejects_foreign_sources_managers_and_registry_errors() {
    let instance = instance(Vec::new());
    let other =
        AssetReference::from_locator(AssetUri::parse("res://prefabs/other.prefab.toml").unwrap());
    let registry = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            instance.prefab.uuid,
            instance.prefab.locator.clone(),
            AssetKind::Prefab,
            "instance-source-test",
        ),
        AssetRegistryEntry::new(
            other.uuid,
            other.locator.clone(),
            AssetKind::Prefab,
            "instance-source-test",
        ),
    ])
    .unwrap();
    let resources = ResourceManager::new();
    for (reference, name) in [(&instance.prefab, "expected"), (&other, "foreign")] {
        resources
            .register_ready(
                ResourceRecord::new(
                    AssetId::from_asset_uuid(reference.uuid),
                    AssetKind::Prefab,
                    reference.locator.clone(),
                ),
                source_asset(&reference.locator, name),
            )
            .unwrap();
    }
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let mut foreign = instance.clone();
    foreign.prefab = other;
    assert!(matches!(
        read.instance_authoring_snapshot(&foreign, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));

    let unrelated = ResourceManager::new();
    unrelated
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                instance.prefab.locator.clone(),
            ),
            source_asset(&instance.prefab.locator, "another manager"),
        )
        .unwrap();
    assert!(matches!(
        read.instance_authoring_snapshot(&instance, &registry, &unrelated),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));

    let missing = AssetRegistryIndex::from_entries(Vec::<AssetRegistryEntry>::new()).unwrap();
    assert!(matches!(
        read.instance_authoring_snapshot(&instance, &missing, &resources),
        Err(PrefabSourceSnapshotError::Reference(_))
    ));
    let wrong_kind = source_registry(&instance.prefab, &instance.prefab.locator, AssetKind::Data);
    assert!(matches!(
        read.instance_authoring_snapshot(&instance, &wrong_kind, &resources),
        Err(PrefabSourceSnapshotError::WrongAssetKind {
            actual: AssetKind::Data
        })
    ));
    assert_eq!(read.source_snapshot().name, "expected");
}

#[test]
fn prefab_instance_source_snapshot_never_adopts_recreated_source_at_the_same_revision() {
    let instance = instance(vec![override_value("root", "opaque.value", 7)]);
    let before = instance.clone();
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    let record = ResourceRecord::new(id, AssetKind::Prefab, locator.clone());
    resources
        .register_ready(record.clone(), source_asset(locator, "before"))
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let retained = read
        .instance_authoring_snapshot(&instance, &registry, &resources)
        .unwrap();

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    assert!(matches!(
        read.instance_authoring_snapshot(&instance, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    resources
        .register_ready(record, source_asset(locator, "replacement"))
        .unwrap();
    assert!(matches!(
        read.instance_authoring_snapshot(&instance, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));

    let fresh = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_ne!(read.version_key(), fresh.version_key());
    assert_eq!(
        read.source_snapshot().revision(),
        fresh.source_snapshot().revision()
    );
    assert_eq!(read.source_snapshot().name, "before");
    assert_eq!(fresh.source_snapshot().name, "replacement");
    assert!(fresh
        .instance_authoring_snapshot(&instance, &registry, &resources)
        .is_ok());
    assert!(std::ptr::eq(
        retained.override_rows[0].override_value,
        &instance.overrides[0]
    ));
    assert_eq!(instance, before);
}

#[test]
fn prefab_source_entity_lookup_uses_full_ids_without_name_or_order_fallback() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    for ids in [[u64::MAX, 0], [0, u64::MAX]] {
        let resources = ResourceManager::new();
        let mut asset = source_asset(locator, "same names");
        asset.scene.entities = ids
            .into_iter()
            .map(|id| source_entity(id, "same entity name", None))
            .collect();
        resources
            .register_ready(
                ResourceRecord::new(
                    AssetId::from_asset_uuid(instance.prefab.uuid),
                    AssetKind::Prefab,
                    locator.clone(),
                ),
                asset,
            )
            .unwrap();
        let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
        for (expected_index, id) in ids.into_iter().enumerate() {
            let PrefabSourceEntityLookup::Unique(entity) = read.entity_by_id(id) else {
                panic!("expected unique entity ID {id}");
            };
            assert_eq!(entity.entity, id);
            assert!(std::ptr::eq(
                entity,
                &read.source_snapshot().scene.entities[expected_index]
            ));
        }
        assert_eq!(read.entity_by_id(42), PrefabSourceEntityLookup::Missing);
    }
}

#[test]
fn prefab_source_entity_lookup_refuses_duplicate_ids_and_preserves_all_rows() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut asset = source_asset(locator, "duplicate IDs");
    asset.scene.entities = vec![
        source_entity(5, "first", None),
        source_entity(5, "second", None),
        source_entity(9, "unique child", Some(5)),
    ];
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            asset,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(read.entity_by_id(5), PrefabSourceEntityLookup::Ambiguous);
    assert_eq!(read.entity_by_id(6), PrefabSourceEntityLookup::Missing);
    assert!(matches!(
        read.entity_by_id(9),
        PrefabSourceEntityLookup::Unique(entity) if entity.entity == 9
    ));
    let rows = read.authoring_snapshot();
    assert_eq!(rows.entity_rows.len(), 3);
    assert_eq!(rows.entity_rows[0].entity.name, "first");
    assert_eq!(rows.entity_rows[1].entity.name, "second");
    assert_eq!(
        rows.entity_rows[2].parent,
        PrefabSourceEntityParent::Ambiguous
    );
}

#[test]
fn prefab_source_read_distinguishes_ready_empty_from_not_resident_and_stale() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    assert!(matches!(
        PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::NotResident { asset_id }) if asset_id == id
    ));
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, locator.clone()),
            source_asset(locator, "ready empty source"),
        )
        .unwrap();
    let empty = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert!(empty.authoring_snapshot().entity_rows.is_empty());
    assert!(empty.authoring_snapshot().diagnostics.is_empty());
    assert_eq!(empty.entity_by_id(0), PrefabSourceEntityLookup::Missing);
    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    assert!(matches!(
        PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::NotResident { asset_id }) if asset_id == id
    ));
    assert!(matches!(
        empty.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
}

#[test]
fn prefab_source_read_does_not_replace_retained_rows_after_remove_recreate() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    let record = ResourceRecord::new(id, AssetKind::Prefab, locator.clone());
    let mut before = source_asset(locator, "before");
    before.scene.entities = vec![source_entity(42, "before", None)];
    resources.register_ready(record.clone(), before).unwrap();
    let retained = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let retained_rows = retained.authoring_snapshot();

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    let mut after = source_asset(locator, "after");
    after.scene.entities = vec![source_entity(u64::MAX, "after", None)];
    resources.register_ready(record, after).unwrap();
    assert!(matches!(
        retained.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(retained_rows.entity_rows[0].entity.entity, 42);
    assert_eq!(
        current.authoring_snapshot().entity_rows[0].entity.entity,
        u64::MAX
    );
    assert_ne!(retained.version_key(), current.version_key());
    assert_eq!(
        retained.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
}

#[test]
fn prefab_source_expected_snapshot_accepts_the_original_key_after_reference_relocation() {
    let instance = instance(Vec::new());
    let current = AssetUri::parse("res://moved/船体.prefab.toml").unwrap();
    let registry = source_registry(&instance.prefab, &current, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, current.clone()),
            source_asset(&current, "relocated resident source"),
        )
        .unwrap();
    let (key, _) = prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();

    let source =
        prefab_source_snapshot_expected(&instance.prefab, &registry, &resources, &key).unwrap();

    assert_eq!(source.name, "relocated resident source");
    assert_eq!(source.uri, current);
    assert_ne!(instance.prefab.locator, source.uri);
    assert_eq!(source.record().id(), id);
}

#[test]
fn prefab_source_expected_snapshot_rejects_a_different_asset_or_resource_manager_key() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let other_resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    for manager in [&resources, &other_resources] {
        manager
            .register_ready(
                ResourceRecord::new(id, AssetKind::Prefab, locator.clone()),
                source_asset(locator, "same UUID and revision, different owner"),
            )
            .unwrap();
    }
    let (key, _) = prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();
    assert!(matches!(
        prefab_source_snapshot_expected(&instance.prefab, &registry, &other_resources, &key),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            asset_id,
            error: ResourceActiveSnapshotError::Stale,
        }) if asset_id == id
    ));

    let other = AssetReference::from_locator(AssetUri::parse("res://other.prefab.toml").unwrap());
    let other_registry = source_registry(&other, &other.locator, AssetKind::Prefab);
    let other_id = AssetId::from_asset_uuid(other.uuid);
    resources
        .register_ready(
            ResourceRecord::new(other_id, AssetKind::Prefab, other.locator.clone()),
            source_asset(&other.locator, "other source"),
        )
        .unwrap();
    assert!(matches!(
        prefab_source_snapshot_expected(&other, &other_registry, &resources, &key),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            asset_id,
            error: ResourceActiveSnapshotError::Stale,
        }) if asset_id == other_id
    ));
}

#[test]
fn prefab_source_expected_snapshot_does_not_adopt_a_removed_or_recreated_source() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    let record = ResourceRecord::new(id, AssetKind::Prefab, locator.clone());
    resources
        .register_ready(record.clone(), source_asset(locator, "before"))
        .unwrap();
    let (key, before) = prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    assert!(matches!(
        prefab_source_snapshot_expected(&instance.prefab, &registry, &resources, &key),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    resources
        .register_ready(record, source_asset(locator, "after"))
        .unwrap();
    assert!(matches!(
        prefab_source_snapshot_expected(&instance.prefab, &registry, &resources, &key),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    let (current_key, current) =
        prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(before.revision(), current.revision());
    assert_eq!(before.name, "before");
    assert_eq!(
        prefab_source_snapshot_expected(&instance.prefab, &registry, &resources, &current_key)
            .unwrap()
            .name,
        "after"
    );
}

#[test]
fn prefab_source_snapshot_uses_uuid_after_relocation_and_returns_the_real_payload_version() {
    let instance = instance(Vec::new());
    let current = AssetUri::parse("res://moved/船体.prefab.toml").unwrap();
    let registry = source_registry(&instance.prefab, &current, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, current.clone()),
            source_asset(&current, "loaded source"),
        )
        .unwrap();

    let (key, source) = prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();

    assert_eq!(source.name, "loaded source");
    assert_eq!(source.uri, current);
    assert_eq!(source.record().id(), id);
    assert_eq!(key.id(), id);
    assert_ne!(instance.prefab.locator, current);
    assert!(resources
        .snapshot_expected::<PrefabMarker, PrefabAsset>(ResourceHandle::new(id), &key)
        .is_ok());
}

#[test]
fn prefab_source_snapshot_does_not_bind_a_different_uuid_that_reuses_the_locator() {
    let instance = instance(Vec::new());
    let replacement = AssetReference::new(
        zircon_runtime::asset::AssetUuid::from_stable_label("different-prefab-source"),
        instance.prefab.locator.clone(),
    );
    let registry = source_registry(&replacement, &replacement.locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(replacement.uuid),
                AssetKind::Prefab,
                replacement.locator.clone(),
            ),
            source_asset(&replacement.locator, "unrelated asset at old URL"),
        )
        .unwrap();

    assert!(matches!(
        prefab_source_snapshot(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::Reference(_))
    ));
}

#[test]
fn prefab_source_snapshot_distinguishes_unloaded_metadata_from_a_missing_reference() {
    let instance = instance(Vec::new());
    let registry = source_registry(
        &instance.prefab,
        &instance.prefab.locator,
        AssetKind::Prefab,
    );
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_record(ResourceRecord::new(
            id,
            AssetKind::Prefab,
            instance.prefab.locator.clone(),
        ))
        .unwrap();

    assert!(matches!(
        prefab_source_snapshot(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::NotResident { asset_id }) if asset_id == id
    ));
}

#[test]
fn prefab_source_snapshot_rejects_wrong_catalog_kind_and_actual_payload_type() {
    let instance = instance(Vec::new());
    let resources = ResourceManager::new();
    let wrong_catalog =
        source_registry(&instance.prefab, &instance.prefab.locator, AssetKind::Data);
    assert!(matches!(
        prefab_source_snapshot(&instance.prefab, &wrong_catalog, &resources),
        Err(PrefabSourceSnapshotError::WrongAssetKind {
            actual: AssetKind::Data
        })
    ));

    let registry = source_registry(
        &instance.prefab,
        &instance.prefab.locator,
        AssetKind::Prefab,
    );
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, instance.prefab.locator.clone()),
            17_u64,
        )
        .unwrap();
    assert!(matches!(
        prefab_source_snapshot(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::KindConflict,
            ..
        })
    ));
}

#[test]
fn prefab_source_snapshot_native_key_rejects_remove_recreate_at_the_same_id_and_revision() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    let record = ResourceRecord::new(id, AssetKind::Prefab, locator.clone());
    resources
        .register_ready(record.clone(), source_asset(locator, "before"))
        .unwrap();
    let (old_key, old_source) =
        prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    resources
        .register_ready(record, source_asset(locator, "after"))
        .unwrap();

    assert!(matches!(
        resources.snapshot_expected::<PrefabMarker, PrefabAsset>(ResourceHandle::new(id), &old_key),
        Err(ResourceActiveSnapshotError::Stale)
    ));
    let (new_key, new_source) =
        prefab_source_snapshot(&instance.prefab, &registry, &resources).unwrap();
    assert_ne!(old_key, new_key);
    assert_eq!(old_source.revision(), new_source.revision());
    assert_eq!(old_source.name, "before");
    assert_eq!(new_source.name, "after");
}

#[test]
fn prefab_source_rows_borrow_the_resident_payload_without_losing_identity_or_components() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut asset = source_asset(locator, "source tree unit fixture");
    let mut child = source_entity(u64::MAX, "同名组件 / 长名称", Some(42));
    let reference =
        AssetReference::from_locator(AssetUri::parse("res://meshes/船体.mesh").unwrap());
    let component = SceneComponentAssetRecord::from_typed(
        "test.opaque.Component",
        "test.opaque.Schema",
        3,
        "test-provider",
        &serde_json::json!({"mesh": reference, "unknown": {"保留": [1, 2, 3]}}),
    )
    .unwrap();
    child.components = vec![component.clone(), component];
    child.active = false;
    asset.scene.entities = vec![child, source_entity(42, "同名组件 / 长名称", None)];
    let before = asset.clone();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    resources
        .register_ready(
            ResourceRecord::new(id, AssetKind::Prefab, locator.clone()),
            asset,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let source = read.source_snapshot().resource().as_ref();
    let snapshot = read.authoring_snapshot();

    assert!(std::ptr::eq(snapshot.source, source));
    assert_eq!(read.version_key().id(), id);
    assert_eq!(snapshot.entity_rows.len(), 2);
    assert!(snapshot.diagnostics.is_empty());
    for (index, row) in snapshot.entity_rows.iter().enumerate() {
        assert_eq!(row.source_index, index);
        assert!(std::ptr::eq(row.entity, &source.scene.entities[index]));
    }
    assert_eq!(snapshot.entity_rows[0].entity.entity, u64::MAX);
    assert_eq!(
        snapshot.entity_rows[0].parent,
        PrefabSourceEntityParent::Resolved { source_index: 1 }
    );
    assert_eq!(
        snapshot.entity_rows[1].parent,
        PrefabSourceEntityParent::Root
    );
    assert_eq!(snapshot.entity_rows[0].entity.components.len(), 2);
    assert_eq!(
        snapshot.entity_rows[0].entity.components[0]
            .references
            .len(),
        1
    );
    assert_eq!(*source, before);
    let revalidated = read
        .revalidate(&instance.prefab, &registry, &resources)
        .unwrap();
    assert_eq!(read.version_key(), revalidated.version_key());
    assert!(std::sync::Arc::ptr_eq(
        read.source_snapshot().resource(),
        revalidated.source_snapshot().resource()
    ));
    assert!(resources
        .snapshot_expected::<PrefabMarker, PrefabAsset>(ResourceHandle::new(id), read.version_key())
        .is_ok());
}

#[test]
fn prefab_source_rows_preserve_duplicate_ids_and_distinguish_missing_and_ambiguous_parents() {
    let locator = AssetUri::parse("res://prefabs/invalid-tree.prefab.toml").unwrap();
    let mut source = source_asset(&locator, "invalid source tree unit fixture");
    source.scene.entities = vec![
        source_entity(5, "duplicate", None),
        source_entity(5, "duplicate", None),
        source_entity(9, "ambiguous parent", Some(5)),
        source_entity(10, "missing parent", Some(999)),
    ];
    let before = source.clone();

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);

    assert_eq!(snapshot.entity_rows.len(), 4);
    assert_eq!(
        snapshot.entity_rows[2].parent,
        PrefabSourceEntityParent::Ambiguous
    );
    assert_eq!(
        snapshot.entity_rows[3].parent,
        PrefabSourceEntityParent::Missing
    );
    assert_eq!(
        snapshot
            .diagnostics
            .iter()
            .map(|value| (value.kind, value.source_indices.clone()))
            .collect::<Vec<_>>(),
        vec![
            (PrefabSourceDiagnosticKind::DuplicateEntityId, vec![0, 1]),
            (PrefabSourceDiagnosticKind::AmbiguousParent, vec![2]),
            (PrefabSourceDiagnosticKind::MissingParent, vec![3]),
        ]
    );
    assert!(snapshot.diagnostics[2].message.contains("999"));
    assert_eq!(source, before);
}

#[test]
fn prefab_source_hierarchy_preserves_sibling_order_and_full_source_identity() {
    let locator = AssetUri::parse("res://prefabs/source-order.prefab.toml").unwrap();
    let mut source = source_asset(&locator, "source hierarchy unit fixture");
    source.scene.entities = vec![
        source_entity(u64::MAX, "同名实体", Some(20)),
        source_entity(10, "同名实体", None),
        source_entity(21, "同名实体", Some(10)),
        source_entity(20, "同名实体", Some(10)),
        source_entity(40, "同名实体", None),
    ];
    let before = source.clone();

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);

    assert_eq!(
        snapshot
            .hierarchy_rows
            .iter()
            .map(|row| (row.source_index, row.depth, row.has_children))
            .collect::<Vec<_>>(),
        vec![
            (1, 0, true),
            (2, 1, false),
            (3, 1, true),
            (0, 2, false),
            (4, 0, false),
        ]
    );
    assert!(snapshot.diagnostics.is_empty());
    assert_eq!(snapshot.entity_rows[0].entity.entity, u64::MAX);
    for (index, row) in snapshot.entity_rows.iter().enumerate() {
        assert_eq!(row.source_index, index);
        assert!(std::ptr::eq(row.entity, &source.scene.entities[index]));
    }
    assert_eq!(source, before);
}

#[test]
fn prefab_source_hierarchy_keeps_invalid_roots_and_their_valid_descendants_visible() {
    let locator = AssetUri::parse("res://prefabs/source-forest.prefab.toml").unwrap();
    let mut source = source_asset(&locator, "invalid hierarchy unit fixture");
    source.scene.entities = vec![
        source_entity(5, "duplicate", None),
        source_entity(5, "duplicate", None),
        source_entity(9, "ambiguous parent", Some(5)),
        source_entity(10, "missing parent", Some(999)),
        source_entity(11, "cycle a", Some(12)),
        source_entity(12, "cycle b", Some(11)),
        source_entity(13, "child of cycle member", Some(11)),
    ];
    let before = source.clone();

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);

    assert_eq!(
        snapshot
            .hierarchy_rows
            .iter()
            .map(|row| (row.source_index, row.depth, row.has_children))
            .collect::<Vec<_>>(),
        vec![
            (0, 0, false),
            (1, 0, false),
            (2, 0, false),
            (3, 0, false),
            (4, 0, true),
            (6, 1, false),
            (5, 0, false),
        ]
    );
    assert_eq!(
        snapshot
            .diagnostics
            .iter()
            .map(|value| (value.kind, value.source_indices.clone()))
            .collect::<Vec<_>>(),
        vec![
            (PrefabSourceDiagnosticKind::DuplicateEntityId, vec![0, 1]),
            (PrefabSourceDiagnosticKind::AmbiguousParent, vec![2]),
            (PrefabSourceDiagnosticKind::MissingParent, vec![3]),
            (PrefabSourceDiagnosticKind::CyclicHierarchy, vec![4, 5]),
        ]
    );
    assert_eq!(source, before);
}

#[test]
fn prefab_source_rows_cut_cycle_edges_without_recursing_through_deep_hierarchies() {
    let locator = AssetUri::parse("res://prefabs/cyclic-tree.prefab.toml").unwrap();
    let mut source = source_asset(&locator, "cyclic source tree unit fixture");
    source.scene.entities = (0..8192_u64)
        .map(|id| source_entity(id, "deep hierarchy", Some(id + 1)))
        .collect();
    // A long tail reaches a two-entity cycle; a separate row has a self-parent.
    source.scene.entities.last_mut().unwrap().parent = Some(8190);
    source
        .scene
        .entities
        .push(source_entity(9000, "self", Some(9000)));

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);

    assert_eq!(snapshot.entity_rows.len(), 8193);
    assert_eq!(
        snapshot.entity_rows[0].parent,
        PrefabSourceEntityParent::Resolved { source_index: 1 }
    );
    for index in [8190, 8191, 8192] {
        assert_eq!(
            snapshot.entity_rows[index].parent,
            PrefabSourceEntityParent::Cyclic
        );
    }
    assert_eq!(
        snapshot
            .diagnostics
            .iter()
            .map(|value| (value.kind, value.source_indices.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                PrefabSourceDiagnosticKind::CyclicHierarchy,
                vec![8190, 8191]
            ),
            (PrefabSourceDiagnosticKind::CyclicHierarchy, vec![8192]),
        ]
    );
    assert_eq!(snapshot.hierarchy_rows.len(), 8193);
    for (depth, row) in snapshot.hierarchy_rows[..8191].iter().enumerate() {
        assert_eq!(row.source_index, 8190 - depth);
        assert_eq!(row.depth, depth);
        assert_eq!(row.has_children, depth < 8190);
    }
    for (row, source_index) in snapshot.hierarchy_rows[8191..].iter().zip([8191, 8192]) {
        assert_eq!(row.source_index, source_index);
        assert_eq!(row.depth, 0);
        assert!(!row.has_children);
    }
    let empty_source = source_asset(&locator, "empty");
    let empty = PrefabSourceAuthoringSnapshot::new(&empty_source);
    assert!(empty.entity_rows.is_empty());
    assert!(empty.hierarchy_rows.is_empty());
}

#[test]
fn prefab_source_component_lookup_uses_full_entity_and_all_native_descriptor_fields() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let reference =
        AssetReference::from_locator(AssetUri::parse("res://meshes/船体.mesh").unwrap());
    let expected = SceneComponentAssetRecord::from_typed(
        "test.opaque.Component",
        "test.opaque.Schema",
        3,
        "test-provider",
        &serde_json::json!({"mesh": reference, "unknown": {"保留": [1, 2, 3]}}),
    )
    .unwrap();
    let descriptor = PrefabSourceComponentDescriptor::from(&expected);
    for reverse in [false, true] {
        let resources = ResourceManager::new();
        let mut asset = source_asset(locator, "opaque component lookup");
        let mut different_type = expected.clone();
        different_type.type_id.push_str(".other");
        let mut different_schema = expected.clone();
        different_schema.schema_id.push_str(".other");
        let mut different_version = expected.clone();
        different_version.schema_version = 4;
        let mut different_provider = expected.clone();
        different_provider.provider_id.push_str(".other");
        let mut selected = source_entity(u64::MAX, "same entity name", None);
        selected.components = vec![
            different_type,
            different_schema,
            different_version,
            different_provider,
            expected.clone(),
        ];
        let mut other = source_entity(0, "same entity name", None);
        other.components = vec![expected.clone()];
        asset.scene.entities = vec![selected, other];
        if reverse {
            asset.scene.entities[0].components.reverse();
            asset.scene.entities.reverse();
        }
        let before = asset.clone();
        resources
            .register_ready(
                ResourceRecord::new(
                    AssetId::from_asset_uuid(instance.prefab.uuid),
                    AssetKind::Prefab,
                    locator.clone(),
                ),
                asset,
            )
            .unwrap();
        let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
        let PrefabSourceComponentLookup::Unique(record) =
            read.component_by_descriptor(u64::MAX, descriptor)
        else {
            panic!("expected unique opaque component");
        };
        let (entity_index, component_index) = if reverse { (1, 0) } else { (0, 4) };
        assert!(std::ptr::eq(
            record,
            &read.source_snapshot().scene.entities[entity_index].components[component_index]
        ));
        assert_eq!(record, &expected);
        assert_eq!(record.references.len(), 1);
        assert_eq!(*read.source_snapshot().resource().as_ref(), before);
    }
}

#[test]
fn prefab_source_component_lookup_distinguishes_missing_and_ambiguous_owners_and_records() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let component = SceneComponentAssetRecord::from_typed(
        "test.Component",
        "test.Schema",
        1,
        "test-provider",
        &serde_json::json!({"value": 1}),
    )
    .unwrap();
    let descriptor = PrefabSourceComponentDescriptor::from(&component);
    let mut duplicate_entity = source_entity(5, "duplicate owner", None);
    duplicate_entity.components = vec![component.clone()];
    let mut duplicate_component = source_entity(8, "duplicate records", None);
    duplicate_component.components = vec![component.clone(), component.clone()];
    let mut mismatch = source_entity(9, "different schema", None);
    let mut changed = component.clone();
    changed.schema_version = 2;
    mismatch.components = vec![changed];
    let mut asset = source_asset(locator, "ambiguous component lookup");
    asset.scene.entities = vec![
        duplicate_entity,
        source_entity(5, "other duplicate owner", None),
        source_entity(7, "no component", None),
        duplicate_component,
        mismatch,
    ];
    let before = asset.clone();
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            asset,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(
        read.component_by_descriptor(5, descriptor),
        PrefabSourceComponentLookup::AmbiguousEntity
    );
    assert_eq!(
        read.component_by_descriptor(42, descriptor),
        PrefabSourceComponentLookup::MissingEntity
    );
    assert_eq!(
        read.component_by_descriptor(7, descriptor),
        PrefabSourceComponentLookup::MissingComponent
    );
    assert_eq!(
        read.component_by_descriptor(8, descriptor),
        PrefabSourceComponentLookup::AmbiguousComponent
    );
    assert_eq!(
        read.component_by_descriptor(9, descriptor),
        PrefabSourceComponentLookup::MissingComponent
    );
    assert_eq!(*read.source_snapshot().resource().as_ref(), before);
}

#[test]
fn prefab_source_component_lookup_cannot_rebind_an_equal_descriptor_after_replacement() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let id = AssetId::from_asset_uuid(instance.prefab.uuid);
    let resource_record = ResourceRecord::new(id, AssetKind::Prefab, locator.clone());
    let component = SceneComponentAssetRecord::from_typed(
        "test.Component",
        "test.Schema",
        1,
        "test-provider",
        &serde_json::json!({"value": 1}),
    )
    .unwrap();
    let descriptor = PrefabSourceComponentDescriptor::from(&component);
    let mut original = source_asset(locator, "before");
    let mut entity = source_entity(42, "same name", None);
    entity.components = vec![component.clone()];
    original.scene.entities = vec![entity.clone()];
    resources
        .register_ready(resource_record.clone(), original)
        .unwrap();
    let retained = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let PrefabSourceComponentLookup::Unique(retained_record) =
        retained.component_by_descriptor(42, descriptor)
    else {
        panic!("expected original component");
    };

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    entity.components[0].payload = serde_json::json!({"value": 2});
    let mut replacement = source_asset(locator, "after");
    replacement.scene.entities = vec![entity];
    resources
        .register_ready(resource_record, replacement)
        .unwrap();
    assert!(matches!(
        retained.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let PrefabSourceComponentLookup::Unique(current_record) =
        current.component_by_descriptor(42, descriptor)
    else {
        panic!("expected replacement component");
    };
    assert_eq!(retained_record.payload["value"], 1);
    assert_eq!(current_record.payload["value"], 2);
    assert_ne!(retained.version_key(), current.version_key());
    assert_eq!(
        retained.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
}

#[test]
fn prefab_source_component_conflicts_locate_all_records_without_merging_opaque_values() {
    let locator = AssetUri::parse("res://prefabs/component-conflicts.prefab.toml").unwrap();
    let make_record = |type_id: &str, value: i32, mesh: &AssetReference| {
        SceneComponentAssetRecord::from_typed(
            type_id,
            "test.opaque.Schema",
            3,
            "unknown-provider",
            &serde_json::json!({"value": value, "mesh": mesh}),
        )
        .unwrap()
    };
    let mesh_a = AssetReference::from_locator(AssetUri::parse("res://meshes/船体.mesh").unwrap());
    let mesh_b =
        AssetReference::from_locator(AssetUri::parse("res://meshes/另一船体.mesh").unwrap());
    let mut entity = source_entity(u64::MAX, "same name", None);
    entity.components = vec![
        make_record("Z.Component", 1, &mesh_a),
        make_record("A.Component", 2, &mesh_a),
        make_record("Z.Component", 3, &mesh_b),
        make_record("A.Component", 4, &mesh_b),
        make_record("Z.Component", 5, &mesh_a),
    ];
    let mut source = source_asset(&locator, "opaque conflicts");
    source.scene.entities = vec![entity];
    let before = source.clone();

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);
    assert_eq!(
        snapshot.component_conflicts,
        vec![
            PrefabSourceComponentConflict {
                entity_source_index: 0,
                component_source_indices: vec![0, 2, 4],
            },
            PrefabSourceComponentConflict {
                entity_source_index: 0,
                component_source_indices: vec![1, 3],
            },
        ]
    );
    assert!(std::ptr::eq(snapshot.source, &source));
    for (index, record) in snapshot.entity_rows[0].entity.components.iter().enumerate() {
        assert!(std::ptr::eq(
            record,
            &source.scene.entities[0].components[index]
        ));
        assert_eq!(record.payload["value"], index + 1);
        assert_eq!(record.references.len(), 1);
    }
    assert_eq!(
        snapshot.entity_rows[0].entity.components[0].references[0],
        mesh_a
    );
    assert_eq!(
        snapshot.entity_rows[0].entity.components[2].references[0],
        mesh_b
    );
    assert!(snapshot.diagnostics.is_empty());
    assert_eq!(source, before);
}

#[test]
fn prefab_source_component_conflicts_use_full_descriptors_and_serialized_owner_positions() {
    let locator = AssetUri::parse("res://prefabs/component-owners.prefab.toml").unwrap();
    let record = SceneComponentAssetRecord::from_typed(
        "test.Component",
        "test.Schema",
        u32::MAX,
        "提供者",
        &serde_json::json!({"unknown": true}),
    )
    .unwrap();
    let mut first = source_entity(0, "same name", None);
    first.components = vec![record.clone()];
    let second = first.clone();
    let mut different_type = record.clone();
    different_type.type_id.push(' ');
    let mut different_schema = record.clone();
    different_schema.schema_id.push(' ');
    let mut different_version = record.clone();
    different_version.schema_version -= 1;
    let mut different_provider = record.clone();
    different_provider.provider_id.push(' ');
    let mut selected = source_entity(u64::MAX, "same name", None);
    selected.components = vec![
        record.clone(),
        different_type,
        different_schema,
        different_version,
        different_provider,
        record,
    ];
    let mut source = source_asset(&locator, "separate serialized owners");
    source.scene.entities = vec![first, second, selected];

    let snapshot = PrefabSourceAuthoringSnapshot::new(&source);
    assert_eq!(
        snapshot.component_conflicts,
        vec![PrefabSourceComponentConflict {
            entity_source_index: 2,
            component_source_indices: vec![0, 5],
        }]
    );
    assert_eq!(snapshot.diagnostics.len(), 1);
    assert_eq!(
        snapshot.diagnostics[0].kind,
        PrefabSourceDiagnosticKind::DuplicateEntityId
    );
    assert_eq!(snapshot.diagnostics[0].source_indices, vec![0, 1]);
    assert!(snapshot
        .entity_rows
        .iter()
        .all(|row| row.parent == PrefabSourceEntityParent::Root));
    assert!(
        PrefabSourceAuthoringSnapshot::new(&source_asset(&locator, "empty"))
            .component_conflicts
            .is_empty()
    );
}

#[test]
fn prefab_source_component_conflicts_keep_the_original_read_after_source_replacement() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let resource_record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    let record = SceneComponentAssetRecord::from_typed(
        "test.Component",
        "test.Schema",
        1,
        "test-provider",
        &serde_json::json!({"value": 1}),
    )
    .unwrap();
    let mut entity = source_entity(42, "same name", None);
    entity.components = vec![record.clone(), record];
    let mut original = source_asset(locator, "before");
    original.scene.entities = vec![entity];
    resources
        .register_ready(resource_record.clone(), original.clone())
        .unwrap();
    let retained = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let rows = retained.authoring_snapshot();
    assert_eq!(
        rows.component_conflicts[0].component_source_indices,
        vec![0, 1]
    );

    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    original.scene.entities[0].components.pop();
    resources.register_ready(resource_record, original).unwrap();
    assert!(matches!(
        retained.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert!(current.authoring_snapshot().component_conflicts.is_empty());
    assert_eq!(rows.entity_rows[0].entity.components.len(), 2);
    assert_eq!(
        rows.component_conflicts[0].component_source_indices,
        vec![0, 1]
    );
    assert_eq!(
        retained.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
    assert_ne!(retained.version_key(), current.version_key());
}

#[test]
fn prefab_source_visible_hierarchy_uses_full_ids_and_queries_only_visible_branches() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    let mut source = source_asset(locator, "unordered native forest");
    source.scene.entities = vec![
        source_entity(20, "同名", Some(10)),
        source_entity(10, "同名", None),
        source_entity(30, "同名", Some(20)),
        source_entity(40, "同名", Some(10)),
        source_entity(u64::MAX, "同名", None),
        source_entity(0, "同名", Some(u64::MAX)),
    ];
    resources.register_ready(record, source).unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let visited = std::cell::RefCell::new(Vec::new());
    let collapsed: Vec<_> = read
        .visible_hierarchy_rows(|entity| {
            visited.borrow_mut().push(entity);
            false
        })
        .map(|row| read.source_snapshot().scene.entities[row.source_index].entity)
        .collect();
    assert_eq!(collapsed, vec![10, u64::MAX]);
    assert_eq!(visited.into_inner(), vec![10, u64::MAX]);

    let visible = |expanded: &[u64]| {
        read.visible_hierarchy_rows(|entity| expanded.contains(&entity))
            .map(|row| {
                (
                    read.source_snapshot().scene.entities[row.source_index].entity,
                    row.depth,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        visible(&[10]),
        vec![(10, 0), (20, 1), (40, 1), (u64::MAX, 0)]
    );
    assert_eq!(
        visible(&[10, 20]),
        vec![(10, 0), (20, 1), (30, 2), (40, 1), (u64::MAX, 0)]
    );
    assert_eq!(
        visible(&[10, 20, u64::MAX]),
        vec![(10, 0), (20, 1), (30, 2), (40, 1), (u64::MAX, 0), (0, 1)]
    );
    assert_eq!(read.source_snapshot().scene.entities[0].entity, 20);
    assert_eq!(read.authoring_snapshot().hierarchy_rows.len(), 6);
}

#[test]
fn prefab_source_visible_hierarchy_retains_invalid_roots_and_cycle_member_children() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    let mut source = source_asset(locator, "ambiguous native forest");
    source.scene.entities = vec![
        source_entity(1, "same", None),
        source_entity(1, "same", None),
        source_entity(2, "same", Some(1)),
        source_entity(3, "same", Some(404)),
        source_entity(4, "same", Some(5)),
        source_entity(5, "same", Some(4)),
        source_entity(6, "same", Some(4)),
    ];
    resources.register_ready(record, source).unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let visible = |expanded: &[u64]| {
        read.visible_hierarchy_rows(|entity| expanded.contains(&entity))
            .map(|row| read.source_snapshot().scene.entities[row.source_index].entity)
            .collect::<Vec<_>>()
    };
    assert_eq!(visible(&[]), vec![1, 1, 2, 3, 4, 5]);
    assert_eq!(visible(&[4]), vec![1, 1, 2, 3, 4, 6, 5]);
    assert_eq!(read.authoring_snapshot().diagnostics.len(), 4);
}

#[test]
fn prefab_source_visible_hierarchy_keeps_ready_empty_and_recreated_reads_distinct() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    resources
        .register_ready(record.clone(), source_asset(locator, "ready empty"))
        .unwrap();
    let retained = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(
        retained
            .visible_hierarchy_rows(|_| panic!("empty source has no expansion target"))
            .count(),
        0
    );
    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    assert!(matches!(
        PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::NotResident { .. })
    ));

    let mut replacement = source_asset(locator, "replacement");
    replacement.scene.entities = vec![
        source_entity(99, "same", None),
        source_entity(100, "same", Some(99)),
    ];
    resources.register_ready(record, replacement).unwrap();
    assert!(matches!(
        retained.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    assert_eq!(retained.visible_hierarchy_rows(|_| true).count(), 0);
    assert_eq!(retained.source_snapshot().name, "ready empty");
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_ne!(retained.version_key(), current.version_key());
    assert_eq!(
        retained.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
    assert_eq!(
        current
            .visible_hierarchy_rows(|_| false)
            .map(|row| current.source_snapshot().scene.entities[row.source_index].entity)
            .collect::<Vec<_>>(),
        vec![99]
    );
    assert_eq!(
        current
            .visible_hierarchy_rows(|_| true)
            .map(|row| current.source_snapshot().scene.entities[row.source_index].entity)
            .collect::<Vec<_>>(),
        vec![99, 100]
    );
}

#[test]
fn prefab_source_filter_keeps_matching_paths_and_prunes_unmatched_children() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut source = source_asset(locator, "native filter paths");
    source.scene.entities = vec![
        source_entity(30, "目标中文", Some(20)),
        source_entity(10, "root", None),
        source_entity(40, "unmatched branch", Some(10)),
        source_entity(20, "ancestor", Some(10)),
        source_entity(u64::MAX, "other root", None),
        source_entity(0, "目标中文", Some(u64::MAX)),
        source_entity(50, "unmatched leaf", Some(40)),
    ];
    let original = source.clone();
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            source,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let visited = std::cell::RefCell::new(Vec::new());
    let filtered = read.filtered_hierarchy_rows(|row| {
        visited.borrow_mut().push(row.source_index);
        row.entity.name == "目标中文"
    });
    assert_eq!(visited.into_inner(), (0..7).collect::<Vec<_>>());
    assert_eq!(
        filtered
            .iter()
            .map(|row| (row.source_index, row.depth, row.has_children))
            .collect::<Vec<_>>(),
        vec![
            (1, 0, true),
            (3, 1, true),
            (0, 2, false),
            (4, 0, true),
            (5, 1, false)
        ]
    );
    assert_eq!(read.source_snapshot().scene.entities[4].entity, u64::MAX);
    let parent_only = read.filtered_hierarchy_rows(|row| row.entity.entity == 20);
    assert_eq!(
        parent_only
            .iter()
            .map(|row| (row.source_index, row.depth, row.has_children))
            .collect::<Vec<_>>(),
        vec![(1, 0, true), (3, 1, false)]
    );
    assert!(read.filtered_hierarchy_rows(|_| false).is_empty());
    assert_eq!(
        read.filtered_hierarchy_rows(|_| true),
        read.authoring_snapshot().hierarchy_rows
    );
    assert_eq!(&**read.source_snapshot(), &original);
}

#[test]
fn prefab_source_filter_preserves_ambiguous_owners_and_cut_cycle_edges() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut source = source_asset(locator, "invalid filter paths");
    source.scene.entities = vec![
        source_entity(1, "target", None),
        source_entity(1, "other duplicate", None),
        source_entity(2, "ambiguous parent", Some(1)),
        source_entity(3, "missing parent", Some(404)),
        source_entity(4, "cycle root", Some(5)),
        source_entity(5, "other cycle root", Some(4)),
        source_entity(6, "valid child", Some(4)),
    ];
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            source,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let filtered_indices = |id| {
        read.filtered_hierarchy_rows(|row| row.entity.entity == id)
            .iter()
            .map(|row| row.source_index)
            .collect::<Vec<_>>()
    };
    assert_eq!(filtered_indices(1), vec![0, 1]);
    assert_eq!(filtered_indices(2), vec![2]);
    assert_eq!(filtered_indices(3), vec![3]);
    assert_eq!(filtered_indices(6), vec![4, 6]);
    assert_eq!(
        read.filtered_hierarchy_rows(|row| row.entity.name == "target")
            .iter()
            .map(|row| row.source_index)
            .collect::<Vec<_>>(),
        vec![0]
    );
    assert!(matches!(
        read.entity_by_id(1),
        PrefabSourceEntityLookup::Ambiguous
    ));
    assert_eq!(read.authoring_snapshot().diagnostics.len(), 4);
    assert_eq!(read.source_snapshot().scene.entities[3].parent, Some(404));
}

#[test]
fn prefab_source_filter_borrows_native_metadata_and_never_refreshes_a_retained_read() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    let mut source = source_asset(locator, "original metadata");
    source.scene.entities = vec![
        source_entity(11, "root", None),
        source_entity(12, "same name", Some(11)),
    ];
    let component = SceneComponentAssetRecord::from_typed(
        "unknown.Component",
        "unknown.Schema",
        u32::MAX,
        "unknown-provider",
        &serde_json::json!({"opaque": [1, "保留", {"value": 3}]}),
    )
    .unwrap();
    source.scene.entities[1].components.push(component.clone());
    resources.register_ready(record.clone(), source).unwrap();
    let retained = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let expected = PrefabSourceComponentDescriptor::from(&component);
    let filtered = retained.filtered_hierarchy_rows(|row| {
        row.entity.components.iter().any(|value| {
            std::ptr::eq(
                value,
                &retained.source_snapshot().scene.entities[1].components[0],
            ) && PrefabSourceComponentDescriptor::from(value) == expected
        })
    });
    assert_eq!(
        filtered
            .iter()
            .map(|row| row.source_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    assert!(matches!(
        PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::NotResident { .. })
    ));
    resources
        .register_ready(record, source_asset(locator, "ready empty replacement"))
        .unwrap();
    assert!(matches!(
        retained.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    assert_eq!(retained.filtered_hierarchy_rows(|_| true).len(), 2);
    assert_eq!(
        retained.source_snapshot().scene.entities[1].components[0],
        component
    );
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert!(current
        .filtered_hierarchy_rows(|_| panic!("empty resident source has no matcher input"))
        .is_empty());
    assert_eq!(
        retained.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
    assert_ne!(retained.version_key(), current.version_key());
}

#[test]
fn prefab_source_visible_filter_queries_only_retained_visible_branches() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut source = source_asset(locator, "filtered expansion");
    source.scene.entities = vec![
        source_entity(30, "目标中文", Some(20)),
        source_entity(10, "root", None),
        source_entity(40, "pruned branch", Some(10)),
        source_entity(20, "ancestor", Some(10)),
        source_entity(u64::MAX, "other root", None),
        source_entity(0, "目标中文", Some(u64::MAX)),
        source_entity(50, "pruned leaf", Some(40)),
    ];
    let original = source.clone();
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            source,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    let matched = std::cell::RefCell::new(Vec::new());
    let expanded = std::cell::RefCell::new(Vec::new());
    let rows = read.visible_filtered_hierarchy_rows(
        |row| {
            matched.borrow_mut().push(row.source_index);
            row.entity.name == "目标中文"
        },
        |id| {
            expanded.borrow_mut().push(id);
            id != 20
        },
    );
    assert_eq!(*matched.borrow(), (0..7).collect::<Vec<_>>());
    assert!(expanded.borrow().is_empty());
    assert_eq!(
        rows.map(|row| (row.source_index, row.depth, row.has_children))
            .collect::<Vec<_>>(),
        vec![(1, 0, true), (3, 1, true), (4, 0, true), (5, 1, false)]
    );
    assert_eq!(*expanded.borrow(), vec![10, 20, u64::MAX]);
    expanded.borrow_mut().clear();
    assert_eq!(
        read.visible_filtered_hierarchy_rows(
            |row| row.entity.entity == 20,
            |id| {
                expanded.borrow_mut().push(id);
                true
            },
        )
        .map(|row| (row.source_index, row.has_children))
        .collect::<Vec<_>>(),
        vec![(1, true), (3, false)]
    );
    assert_eq!(*expanded.borrow(), vec![10]);
    assert_eq!(
        read.visible_filtered_hierarchy_rows(|_| true, |id| id != 20)
            .collect::<Vec<_>>(),
        read.visible_hierarchy_rows(|id| id != 20)
            .collect::<Vec<_>>()
    );
    assert!(read
        .visible_filtered_hierarchy_rows(|_| false, |_| panic!("no retained branch"))
        .next()
        .is_none());
    assert_eq!(&**read.source_snapshot(), &original);
}

#[test]
fn prefab_source_visible_filter_keeps_ambiguity_and_original_queued_read() {
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(instance.prefab.uuid),
        AssetKind::Prefab,
        locator.clone(),
    );
    let mut source = source_asset(locator, "invalid filtered expansion");
    source.scene.entities = vec![
        source_entity(1, "target", None),
        source_entity(1, "other duplicate", None),
        source_entity(2, "ambiguous parent", Some(1)),
        source_entity(3, "missing parent", Some(404)),
        source_entity(4, "cycle root", Some(5)),
        source_entity(5, "other cycle root", Some(4)),
        source_entity(6, "valid child", Some(4)),
        source_entity(7, "valid leaf", Some(6)),
    ];
    resources.register_ready(record.clone(), source).unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert_eq!(
        read.visible_filtered_hierarchy_rows(
            |row| row.entity.name == "target",
            |_| panic!("filtered duplicate remains a leaf"),
        )
        .map(|row| row.source_index)
        .collect::<Vec<_>>(),
        vec![0]
    );
    assert!(matches!(
        read.entity_by_id(1),
        PrefabSourceEntityLookup::Ambiguous
    ));
    let expanded = std::cell::RefCell::new(Vec::new());
    assert_eq!(
        read.visible_filtered_hierarchy_rows(
            |row| row.entity.entity == 7,
            |id| {
                expanded.borrow_mut().push(id);
                false
            },
        )
        .map(|row| (row.source_index, row.depth, row.has_children))
        .collect::<Vec<_>>(),
        vec![(4, 0, true)]
    );
    assert_eq!(expanded.into_inner(), vec![4]);
    let queued = read.visible_filtered_hierarchy_rows(|row| row.entity.entity == 7, |_| true);
    assert!(resources.remove_by_locator(locator).unwrap().is_some());
    resources
        .register_ready(record, source_asset(locator, "ready empty replacement"))
        .unwrap();
    assert!(matches!(
        read.revalidate(&instance.prefab, &registry, &resources),
        Err(PrefabSourceSnapshotError::SnapshotRejected {
            error: ResourceActiveSnapshotError::Stale,
            ..
        })
    ));
    assert_eq!(
        queued
            .map(|row| (row.source_index, row.depth))
            .collect::<Vec<_>>(),
        vec![(4, 0), (6, 1), (7, 2)]
    );
    let current = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();
    assert!(current
        .visible_filtered_hierarchy_rows(
            |_| panic!("empty resident source has no matcher input"),
            |_| panic!("empty resident source has no branch"),
        )
        .next()
        .is_none());
    assert_eq!(
        read.source_snapshot().revision(),
        current.source_snapshot().revision()
    );
    assert_ne!(read.version_key(), current.version_key());
    assert_eq!(read.authoring_snapshot().diagnostics.len(), 4);
}

fn source_entity(entity: u64, name: &str, parent: Option<u64>) -> SceneEntityAsset {
    serde_json::from_value(serde_json::json!({
        "entity": entity,
        "name": name,
        "parent": parent,
        "transform": TransformAsset::default(),
    }))
    .unwrap()
}

fn source_registry(
    reference: &AssetReference,
    current: &AssetUri,
    kind: AssetKind,
) -> AssetRegistryIndex {
    AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        reference.uuid,
        current.clone(),
        kind,
        "prefab-source-adapter-test",
    )])
    .unwrap()
}

fn source_asset(uri: &AssetUri, name: &str) -> PrefabAsset {
    PrefabAsset {
        uri: uri.clone(),
        name: name.to_string(),
        scene: SceneAsset {
            entities: Vec::new(),
        },
        exposed_properties: Vec::new(),
    }
}

fn instance(overrides: Vec<PrefabPropertyOverrideAsset>) -> PrefabInstanceAsset {
    PrefabInstanceAsset {
        prefab: AssetReference::from_locator(
            AssetUri::parse("res://prefabs/ship.prefab.toml").expect("prefab URI"),
        ),
        local_transform: TransformAsset::default(),
        overrides,
    }
}

fn override_value(entity: &str, property: &str, value: i32) -> PrefabPropertyOverrideAsset {
    PrefabPropertyOverrideAsset {
        entity_path: entity.to_string(),
        property_path: property.to_string(),
        value: serde_json::json!(value),
    }
}
