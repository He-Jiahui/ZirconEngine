//! R05 C07 identity/catalog counterparts; use ZE's existing ResourceManager authority.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    AssetUuid, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
    ResourceManagementQuery, ResourceManager, ResourceMutationBatch, ResourceReadinessState,
    ResourceRecord, ResourceState, TextureMarker,
};

#[derive(Debug, PartialEq, Eq)]
struct TestPayload(&'static str);

fn record(label: &str, path: &str, kind: ResourceKind) -> ResourceRecord {
    ResourceRecord::new(
        ResourceId::from_asset_uuid(AssetUuid::from_stable_label(label)),
        kind,
        ResourceLocator::parse(path).unwrap(),
    )
}

// Unreal Engine/Source/Runtime/CoreUObject/Tests/ObjectPtrTest.cpp:
// CoreUObject::TObjectPtr::Move and ::GetTypeHash retain references/map keys after a move.
// ZE keeps authored UUIDs and explicitly renames each catalog member in one existing batch.
#[test]
fn unreal_equivalent_move_preserves_root_and_subasset_handle_identity() {
    let manager = ResourceManager::new();
    let root = record("R05/root", "res://models/robot.glb", ResourceKind::Model);
    let mesh = record(
        "R05/mesh0",
        "res://models/robot.glb#mesh0",
        ResourceKind::Mesh,
    );
    let root_handle = ResourceHandle::<ModelMarker>::new(root.id);
    let mesh_handle = ResourceHandle::<MeshMarker>::new(mesh.id);
    let cached_by_id = HashMap::from([(root.id, "root"), (mesh.id, "mesh0")]);
    manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_ready(root.clone(), TestPayload("root"))
                .upsert_ready(mesh.clone(), TestPayload("mesh0")),
        )
        .unwrap();
    let root_payload = manager.get::<_, TestPayload>(root_handle).unwrap();
    let mesh_payload = manager.get::<_, TestPayload>(mesh_handle).unwrap();
    let previous = manager.projection_snapshot();
    let destination = ResourceLocator::parse("res://actors/robot.glb").unwrap();
    let mesh_destination = ResourceLocator::parse("res://actors/robot.glb#mesh0").unwrap();

    manager
        .commit(
            ResourceMutationBatch::new()
                .rename(root.primary_locator.clone(), destination.clone())
                .rename(mesh.primary_locator.clone(), mesh_destination.clone()),
        )
        .unwrap();

    for (id, old_path, new_path) in [
        (root.id, &root.primary_locator, &destination),
        (mesh.id, &mesh.primary_locator, &mesh_destination),
    ] {
        let current = manager.projection_snapshot();
        let by_id = current.management().row_by_id(id).unwrap();
        let by_path = current
            .management()
            .row_by_locator(&new_path.to_string())
            .unwrap();
        assert!(Arc::ptr_eq(&by_id, &by_path));
        assert!(current
            .management()
            .row_by_locator(&old_path.to_string())
            .is_none());
        assert_eq!(
            current.readiness().row(id).unwrap().record.primary_locator,
            *new_path
        );
        assert_eq!(
            previous
                .management()
                .row_by_id(id)
                .unwrap()
                .primary_locator
                .as_ref(),
            old_path.to_string()
        );
    }
    let moved_root_id = manager.registry().get_by_locator(&destination).unwrap().id;
    let moved_mesh_id = manager
        .registry()
        .get_by_locator(&mesh_destination)
        .unwrap()
        .id;
    assert_eq!(root_handle.id(), moved_root_id);
    assert_eq!(mesh_handle.id(), moved_mesh_id);
    assert_eq!(cached_by_id.get(&moved_root_id), Some(&"root"));
    assert_eq!(cached_by_id.get(&moved_mesh_id), Some(&"mesh0"));
    assert!(Arc::ptr_eq(
        &root_payload,
        &manager.get::<_, TestPayload>(root_handle).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &mesh_payload,
        &manager.get::<_, TestPayload>(mesh_handle).unwrap()
    ));
}

// Unreal Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryStateTest.cpp:
// FEnumerateAssetsPerformance constructs distractors and asserts exactly one filtered row.
// Adapt the selection to ZE's kind/state query; this asserts identity without timing limits.
#[test]
fn unreal_equivalent_catalog_filter_returns_the_exact_published_identity() {
    let manager = ResourceManager::new();
    let selected = record(
        "R05/selected",
        "res://textures/selected.png",
        ResourceKind::Texture,
    );
    let pending = record(
        "R05/pending",
        "res://textures/pending.png",
        ResourceKind::Texture,
    );
    let other_kind = record(
        "R05/other-kind",
        "res://models/selected.glb",
        ResourceKind::Model,
    );
    manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_ready(selected.clone(), TestPayload("selected"))
                .upsert_lazy(pending.clone())
                .upsert_ready(other_kind, TestPayload("other-kind")),
        )
        .unwrap();
    let published = manager.projection_snapshot();
    let mut scan = published.management().scan(ResourceManagementQuery {
        kind: Some(ResourceKind::Texture),
        state: Some(ResourceState::Ready),
    });

    let row = scan.next_row().expect("one matching texture");

    assert_eq!(row.id, selected.id);
    assert!(scan.next_row().is_none());
    assert!(scan.is_complete());
    assert!(Arc::ptr_eq(
        &row,
        &published
            .management()
            .row_by_locator(&selected.primary_locator.to_string())
            .unwrap()
    ));
    let readiness = published.readiness().row(row.id).unwrap();
    assert_eq!(readiness.record.id, row.id);
    assert_eq!(readiness.record.revision, row.revision);
    assert_eq!(
        readiness.typed_load_state::<TestPayload>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        manager
            .get::<TextureMarker, TestPayload>(ResourceHandle::new(row.id))
            .unwrap()
            .0,
        "selected"
    );
    assert!(published.management().row_by_id(pending.id).is_some());
    assert_eq!(
        published.readiness().row(pending.id).unwrap().load_state,
        ResourceReadinessState::Loading
    );
}

// Unreal Engine/Source/Runtime/CoreUObject/Tests/ObjectPropertyTest.cpp:
// UE::CoreUObject::FObjectProperty::ParseObjectPropertyValue creates a catalog-only entry
// and obtains an unresolved reference. A ZE typed handle likewise need not own a payload.
#[test]
fn unreal_equivalent_catalog_identity_does_not_imply_resident_payload() {
    let manager = ResourceManager::new();
    let catalog_record = record(
        "R05/catalog-only",
        "res://models/catalog-only.glb",
        ResourceKind::Model,
    )
    .with_state(ResourceState::Ready);
    let handle = manager
        .register_record(catalog_record.clone())
        .unwrap()
        .typed::<ModelMarker>()
        .unwrap();

    let published = manager.projection_snapshot();

    assert_eq!(handle.id(), catalog_record.id);
    assert_eq!(
        published.management().row_by_id(handle.id()).unwrap().state,
        ResourceState::Ready
    );
    assert!(published
        .readiness()
        .contains_kind(handle.id(), ResourceKind::Model));
    assert_eq!(
        published.readiness().row(handle.id()).unwrap().load_state,
        ResourceReadinessState::NotLoaded
    );
    assert!(manager.get::<_, TestPayload>(handle).is_none());
    assert!(manager.snapshot::<_, TestPayload>(handle).is_none());
}
