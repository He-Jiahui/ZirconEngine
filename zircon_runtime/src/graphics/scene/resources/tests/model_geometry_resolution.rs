use super::*;

#[test]
fn runtime93_borrowed_mesh_locator_cache_deduplicates_equal_locators() {
    let first = crate::asset::AssetUri::parse("res://meshes/shared.zmesh").unwrap();
    let second = crate::asset::AssetUri::parse("res://meshes/shared.zmesh").unwrap();
    let mut mesh_assets = HashMap::<&crate::asset::AssetUri, u32>::new();

    mesh_assets.entry(&first).or_insert(11);
    mesh_assets.entry(&second).or_insert(22);

    assert_eq!(mesh_assets.len(), 1);
    assert_eq!(mesh_assets[&first], 11);
}

#[test]
fn composite_geometry_revision_changes_when_external_mesh_revision_changes() {
    let model_id = ResourceId::from_stable_label("tests/model-with-external-mesh");
    let original = dependency_state(7);
    let reloaded = dependency_state(8);

    let original_revision = model_geometry_revision(model_id, 3, &[original]);
    let reloaded_revision = model_geometry_revision(model_id, 3, &[reloaded]);

    assert_ne!(original_revision, reloaded_revision);
}

fn dependency_state(revision: u64) -> ModelMeshDependencyState {
    ModelMeshDependencyState {
        locator: crate::asset::AssetUri::parse("res://meshes/external.zmesh").unwrap(),
        resource_id: Some(ResourceId::from_stable_label("tests/external-mesh")),
        revision: Some(revision),
        state: Some(ResourceState::Ready),
    }
}

#[test]
fn m6_model_mesh_resolution_uses_one_active_capture_and_rejects_drift() {
    use crate::core::resource::{
        MeshMarker, ResourceActiveSnapshotError, ResourceHandle, ResourceKind, ResourceRecord,
        ResourceSourceMember, ResourceSourceOutcome, ResourceSourcePublication,
    };
    use std::sync::Arc;
    let manager = ProjectAssetManager::default();
    let resources = manager.resource_manager();
    let uri = crate::asset::AssetUri::parse("res://models/m6-composite.glb").unwrap();
    let mesh_uri = crate::asset::AssetUri::parse("res://models/m6-composite.glb#Mesh0").unwrap();
    let root_id = ResourceId::from_locator(&uri);
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    let mesh = crate::asset::MeshAsset::new(
        mesh_uri.clone(),
        crate::core::framework::render::RenderMeshTopology::TriangleList,
        std::collections::BTreeMap::from([
            (
                crate::asset::MESH_ATTRIBUTE_POSITION.to_string(),
                crate::asset::MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [2.0, 0.0, 0.0],
                    [0.0, 2.0, 0.0],
                ]),
            ),
            (
                crate::asset::MESH_ATTRIBUTE_NORMAL.to_string(),
                crate::asset::MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                crate::asset::MESH_ATTRIBUTE_UV0.to_string(),
                crate::asset::MeshAttributeValues::Float32x2(vec![
                    [0.0, 0.0],
                    [1.0, 0.0],
                    [0.0, 1.0],
                ]),
            ),
        ]),
        Some(crate::asset::MeshIndices::U32(vec![0, 1, 2])),
    )
    .unwrap();
    let mut inline = mesh.to_model_primitive().unwrap();
    for vertex in &mut inline.vertices {
        vertex.position = [0.25; 3];
    }
    inline.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
    let model = ModelAsset {
        uri: uri.clone(),
        primitives: vec![inline],
    };
    let root = ResourceRecord::new(root_id, ResourceKind::Model, uri)
        .with_state(ResourceState::Ready)
        .with_source_hash("a")
        .with_dependency_ids(vec![mesh_id]);
    let child = ResourceRecord::new(mesh_id, ResourceKind::Mesh, mesh_uri)
        .with_state(ResourceState::Ready)
        .with_source_hash("a");
    resources
        .register_ready(root.clone(), model.clone())
        .unwrap();
    resources
        .register_ready(child.clone(), mesh.clone())
        .unwrap();
    let publication = ResourceSourcePublication::capture(
        root_id,
        Vec::new(),
        vec![
            ResourceSourceMember {
                record: root.clone(),
                payload: Some(Arc::new(model.clone())),
            },
            ResourceSourceMember {
                record: child.clone(),
                payload: Some(Arc::new(mesh.clone())),
            },
        ],
        ResourceSourceOutcome::CompleteSuccess,
        resources.projection_snapshot().active(),
    );
    resources
        .commit(crate::core::resource::ResourceMutationBatch::new().publish_source(publication))
        .unwrap();
    let captured = resources.projection_snapshot();
    let resolved = resolve_model_geometry(&manager, &model, &captured).unwrap();
    assert_eq!(resolved.local_bounds.max, [2.0, 2.0, 0.0]);
    assert_eq!(
        resolved.dependency_keys,
        vec![(mesh_id, captured.active().key(mesh_id))]
    );

    // A failed primary source may remove its candidate child, while the active child remains real.
    let failure = ResourceSourcePublication::capture(
        root_id,
        vec![root_id, mesh_id],
        vec![ResourceSourceMember {
            record: root.clone().with_state(ResourceState::Error),
            payload: None,
        }],
        ResourceSourceOutcome::Failed,
        captured.active(),
    );
    resources
        .commit(crate::core::resource::ResourceMutationBatch::new().publish_source(failure))
        .unwrap();
    assert!(resources.registry().get(mesh_id).is_none());
    let failed = resources.projection_snapshot();
    let retained = resolve_model_geometry(&manager, &model, &failed).unwrap();
    assert_eq!(retained.local_bounds, resolved.local_bounds);
    assert_eq!(retained.dependency_keys, resolved.dependency_keys);
    assert_eq!(
        resources.ref_count(mesh_id),
        Some(0),
        "captured metadata and snapshots create no lease"
    );

    resources
        .register_ready(child.clone().with_source_hash("b"), mesh.clone())
        .unwrap();
    assert!(matches!(
        resolve_model_geometry(&manager, &model, &captured),
        Err(ResourceActiveSnapshotError::Stale)
    ));
    let fresh = resources.projection_snapshot();
    assert_ne!(fresh.active().key(mesh_id), captured.active().key(mesh_id));
    assert!(resources
        .snapshot_expected::<_, crate::asset::MeshAsset>(
            ResourceHandle::<MeshMarker>::new(mesh_id),
            &captured.active().key(mesh_id).unwrap(),
        )
        .is_err());
    resources
        .register_ready(child.clone().with_source_hash("wrong-type"), 7_u8)
        .unwrap();
    assert!(matches!(
        resolve_model_geometry(&manager, &model, &resources.projection_snapshot()),
        Err(ResourceActiveSnapshotError::KindConflict)
    ));

    resources.remove_by_locator(&child.primary_locator).unwrap();
    let absent = resources.projection_snapshot();
    let fallback = resolve_model_geometry(&manager, &model, &absent).unwrap();
    assert_eq!(
        fallback.primitives[0].vertices[0].position, [0.25; 3],
        "only a captured absence may use the original inline fallback"
    );
}
