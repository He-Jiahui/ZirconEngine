use std::collections::HashSet;

use serde_json::json;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::resource::{
    AssetUuid, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceId, ResourceKind,
    ResourceLocator, ResourceManager, ResourceRecord,
};

use super::{bone, install, SkeletonReference};
use crate::core::editing::skeleton_reference::{
    GltfSkinDataError, GltfSkinReference, GltfSkinReferenceError,
};

fn locator(label: &str) -> ResourceLocator {
    ResourceLocator::parse(&format!("res://skin-reference/骑士.glb#{label}")).unwrap()
}

fn source_record(kind: ResourceKind, locator: ResourceLocator) -> ResourceRecord {
    // The registry's supplied sidecar identity deliberately differs from locator-derived IDs.
    let id = ResourceId::from_asset_uuid(AssetUuid::from_stable_label(&format!(
        "skin-reference-test-sidecar:{locator}"
    )));
    assert_ne!(id, ResourceId::from_locator(&locator));
    ResourceRecord::new(id, kind, locator)
}

fn skeleton(resources: &ResourceManager, skin: usize) -> SkeletonReference {
    let record = source_record(
        ResourceKind::AnimationSkeleton,
        locator(&format!("Skin{skin}/Skeleton")),
    );
    let version = install(
        resources,
        record,
        vec![
            bone("Head/头部", Some(2)),
            bone("Head/头部", Some(0)),
            bone("根骨骼", None),
        ],
    );
    SkeletonReference::read(resources, &version).unwrap()
}

fn data(reference: &SkeletonReference, skin: usize) -> DataAsset {
    // Explicit unit payload matching the current importer schema; no importer/mesh decode claimed.
    let joints = [10, 21, 11]
        .into_iter()
        .enumerate()
        .map(|(slot, node)| {
            let key = reference.key(slot as u32).unwrap();
            json!({
                "node_index": node,
                "node": locator(&format!("Node{node}")).to_string(),
                "bone_name": reference.bone(&key).unwrap().name,
            })
        })
        .collect::<Vec<_>>();
    let canonical_json = json!({
        "kind": "gltf_skin",
        "skin_index": skin,
        "skeleton_asset": reference.record().primary_locator.to_string(),
        "joint_count": joints.len(),
        "joints": joints,
        "inverse_bind_matrices": locator(&format!("Skin{skin}/InverseBindMatrices")).to_string(),
        "inverse_bind_matrix_count": 3,
    });
    DataAsset {
        uri: locator(&format!("Skin{skin}")),
        format: DataAssetFormat::Json,
        text: serde_json::to_string(&canonical_json).unwrap(),
        canonical_json,
    }
}

fn install_data(resources: &ResourceManager, data: DataAsset) -> ResourceActiveVersionKey {
    let record = source_record(ResourceKind::Data, data.uri.clone());
    let id = record.id;
    resources.register_ready(record, data).unwrap();
    resources.projection_snapshot().active().key(id).unwrap()
}

fn metadata_error(
    resources: &ResourceManager,
    reference: &SkeletonReference,
    data: DataAsset,
) -> GltfSkinDataError {
    let version = install_data(resources, data);
    match GltfSkinReference::read(resources, &version, reference).unwrap_err() {
        GltfSkinReferenceError::Metadata(error) => error,
        error => panic!("unexpected snapshot error: {error}"),
    }
}

#[test]
fn skin_mapping_preserves_original_slots_nonfirst_skin_and_sidecar_ids() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let version = install_data(&resources, data(&reference, 4));
    let skin = GltfSkinReference::read(&resources, &version, &reference).unwrap();
    assert_eq!(skin.resource_version(), &version);
    assert_eq!(skin.record().id, version.id());
    assert_eq!(skin.skin_index(), 4);
    assert_eq!(reference.joint_count(), 3);
    assert_eq!(
        skin.joints()
            .iter()
            .map(|joint| joint.node_index)
            .collect::<Vec<_>>(),
        [10, 21, 11]
    );
    for (slot, node) in [10, 21, 11].into_iter().enumerate() {
        let key = reference.key(slot as u32).unwrap();
        assert_eq!(skin.key_for_node(node), Some(&key));
        let joint = skin.joint_for_key(&key).unwrap();
        assert_eq!(joint.node, locator(&format!("Node{node}")));
        assert_eq!(joint.key.joint_index(), slot as u32);
    }
    let rows = reference.rows_for_name_query("head", &HashSet::new());
    assert_eq!(
        rows.iter()
            .map(|row| row.key.joint_index())
            .collect::<Vec<_>>(),
        [2, 0, 1]
    );
    assert_eq!(skin.key_for_node(10), reference.key(0).as_ref());
    assert!(skin.key_for_node(999).is_none());
}

#[test]
fn skin_mapping_rejects_foreign_skeleton_and_inconsistent_joint_counts() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let foreign = skeleton(&resources, 5);
    assert!(matches!(
        metadata_error(&resources, &foreign, data(&reference, 4)),
        GltfSkinDataError::SkeletonIdentity
    ));
    assert!(matches!(
        metadata_error(&resources, &foreign, data(&foreign, 4)),
        GltfSkinDataError::SubassetIdentity
    ));
    let original = data(&reference, 4);
    for (declared, count) in [(2, 3), (3, 2), (2, 2), (0, 0)] {
        let mut changed = original.clone();
        changed.canonical_json["joint_count"] = json!(declared);
        changed.canonical_json["joints"]
            .as_array_mut()
            .unwrap()
            .truncate(count);
        assert!(matches!(
            metadata_error(&resources, &reference, changed),
            GltfSkinDataError::JointCount { .. }
        ));
    }
}

#[test]
fn skin_mapping_rejects_invalid_json_kind_and_payload_identity() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let original = data(&reference, 4);
    let mut changed = original.clone();
    changed.canonical_json["skin_index"] = json!(-1);
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::Json(_)
    ));
    let mut changed = original.clone();
    changed.canonical_json["kind"] = json!("gltf_inverse_bind_matrices");
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::Kind { .. }
    ));
    let mut changed = original.clone();
    changed.format = DataAssetFormat::Text;
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::PayloadIdentity
    ));
    let record = source_record(ResourceKind::Data, original.uri.clone());
    let id = record.id;
    let mut changed = original;
    changed.uri = locator("Skin5");
    resources.register_ready(record, changed).unwrap();
    let version = resources.projection_snapshot().active().key(id).unwrap();
    assert!(matches!(
        GltfSkinReference::read(&resources, &version, &reference),
        Err(GltfSkinReferenceError::Metadata(
            GltfSkinDataError::PayloadIdentity
        ))
    ));
}

#[test]
fn skin_mapping_rejects_duplicate_nodes_name_disagreement_and_wrong_source() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let original = data(&reference, 4);
    let mut changed = original.clone();
    changed.canonical_json["joints"][1] = changed.canonical_json["joints"][0].clone();
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::DuplicateNode { node_index: 10 }
    ));
    let mut changed = original.clone();
    changed.canonical_json["joints"]
        .as_array_mut()
        .unwrap()
        .swap(0, 2);
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::BoneName { joint_slot: 0 }
    ));
    for node in [
        "res://another.glb#Node10",
        "res://skin-reference/骑士.glb#Node99",
    ] {
        let mut changed = original.clone();
        changed.canonical_json["joints"][0]["node"] = json!(node);
        assert!(matches!(
            metadata_error(&resources, &reference, changed),
            GltfSkinDataError::SubassetIdentity
        ));
    }
    let mut changed = original;
    changed.canonical_json["joints"][0]["node"] = json!("missing-scheme");
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::Locator(_)
    ));
}

#[test]
fn skin_inverse_bind_metadata_does_not_load_matrices_or_invent_identity_payloads() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let original = data(&reference, 4);
    let version = install_data(&resources, original.clone());
    let skin = GltfSkinReference::read(&resources, &version, &reference).unwrap();
    assert_eq!(
        skin.inverse_bind_matrices(),
        Some(&locator("Skin4/InverseBindMatrices"))
    );
    assert_eq!(skin.inverse_bind_matrix_count(), 3);
    assert!(resources
        .projection_snapshot()
        .active()
        .id_for_locator(&locator("Skin4/InverseBindMatrices"))
        .is_none());
    let mut absent = original.clone();
    absent.canonical_json["inverse_bind_matrices"] = json!(null);
    absent.canonical_json["inverse_bind_matrix_count"] = json!(0);
    let version = install_data(&resources, absent.clone());
    let skin = GltfSkinReference::read(&resources, &version, &reference).unwrap();
    assert!(skin.inverse_bind_matrices().is_none());
    assert_eq!(skin.inverse_bind_matrix_count(), 0);
    for mut changed in [original.clone(), absent] {
        changed.canonical_json["inverse_bind_matrix_count"] = json!(2);
        assert!(matches!(
            metadata_error(&resources, &reference, changed),
            GltfSkinDataError::InverseBindCount { .. }
        ));
    }
    let mut changed = original;
    changed.canonical_json["inverse_bind_matrices"] =
        json!(locator("Skin5/InverseBindMatrices").to_string());
    assert!(matches!(
        metadata_error(&resources, &reference, changed),
        GltfSkinDataError::SubassetIdentity
    ));
}

#[test]
fn skin_expected_snapshot_rejects_wrong_kind_and_wrong_payload_type() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let original = data(&reference, 4);
    let record = source_record(ResourceKind::Mesh, original.uri.clone());
    let id = record.id;
    resources.register_ready(record, original.clone()).unwrap();
    let version = resources.projection_snapshot().active().key(id).unwrap();
    assert!(matches!(
        GltfSkinReference::read(&resources, &version, &reference),
        Err(GltfSkinReferenceError::Snapshot(
            ResourceActiveSnapshotError::KindConflict
        ))
    ));
    resources.remove_by_locator(&original.uri).unwrap().unwrap();
    let record = source_record(ResourceKind::Data, original.uri);
    let id = record.id;
    resources
        .register_ready(record, "not DataAsset".to_owned())
        .unwrap();
    let version = resources.projection_snapshot().active().key(id).unwrap();
    assert!(matches!(
        GltfSkinReference::read(&resources, &version, &reference),
        Err(GltfSkinReferenceError::Snapshot(
            ResourceActiveSnapshotError::KindConflict
        ))
    ));
}

#[test]
fn skin_replacement_and_recreation_reject_old_versions_without_losing_retained_mapping() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let original_data = data(&reference, 4);
    let version = install_data(&resources, original_data.clone());
    let original = GltfSkinReference::read(&resources, &version, &reference).unwrap();
    let mut changed = original_data.clone();
    changed.canonical_json["joints"][0]["node_index"] = json!(90);
    changed.canonical_json["joints"][0]["node"] = json!(locator("Node90").to_string());
    resources
        .store_payload(version.id(), original.record().revision, changed)
        .unwrap();
    let replaced_version = resources
        .projection_snapshot()
        .active()
        .key(version.id())
        .unwrap();
    assert_ne!(version, replaced_version);
    assert!(matches!(
        GltfSkinReference::read(&resources, &version, &reference),
        Err(GltfSkinReferenceError::Snapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
    let replacement = GltfSkinReference::read(&resources, &replaced_version, &reference).unwrap();
    assert!(replacement.key_for_node(10).is_none());
    assert_eq!(replacement.key_for_node(90), reference.key(0).as_ref());
    assert_eq!(original.key_for_node(10), reference.key(0).as_ref());
    resources
        .remove_by_locator(&original_data.uri)
        .unwrap()
        .unwrap();
    let recreated_version = install_data(&resources, original_data);
    let recreated = GltfSkinReference::read(&resources, &recreated_version, &reference).unwrap();
    assert_eq!(original.record().revision, recreated.record().revision);
    assert_ne!(version, recreated_version);
    assert_ne!(replaced_version, recreated_version);
    assert!(matches!(
        GltfSkinReference::read(&resources, &replaced_version, &reference),
        Err(GltfSkinReferenceError::Snapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
    assert_eq!(recreated.key_for_node(10), reference.key(0).as_ref());
}

#[test]
fn skin_mapping_rejects_foreign_and_replaced_skeleton_keys_but_retains_old_read_view() {
    let resources = ResourceManager::new();
    let reference = skeleton(&resources, 4);
    let skin_version = install_data(&resources, data(&reference, 4));
    let original = GltfSkinReference::read(&resources, &skin_version, &reference).unwrap();
    let key = reference.key(0).unwrap();
    let foreign = skeleton(&resources, 5);
    assert!(original.joint_for_key(&foreign.key(0).unwrap()).is_none());
    let source = reference.record().clone();
    let asset = AnimationSkeletonAsset {
        name: None,
        bones: (0..reference.joint_count())
            .map(|slot| {
                reference
                    .bone(&reference.key(slot as u32).unwrap())
                    .unwrap()
                    .clone()
            })
            .collect(),
    };
    resources
        .store_payload(source.id, source.revision, asset.clone())
        .unwrap();
    let version = resources
        .projection_snapshot()
        .active()
        .key(source.id)
        .unwrap();
    let replaced = SkeletonReference::read(&resources, &version).unwrap();
    let mapping = GltfSkinReference::read(&resources, &skin_version, &replaced).unwrap();
    assert!(mapping.joint_for_key(&key).is_none());
    let replacement_key = replaced.key(0).unwrap();
    assert!(mapping.joint_for_key(&replacement_key).is_some());
    resources
        .remove_by_locator(&source.primary_locator)
        .unwrap()
        .unwrap();
    let recreated_version = install(&resources, source, asset.bones);
    let recreated = SkeletonReference::read(&resources, &recreated_version).unwrap();
    let mapping = GltfSkinReference::read(&resources, &skin_version, &recreated).unwrap();
    assert!(mapping.joint_for_key(&key).is_none());
    assert!(mapping.joint_for_key(&replacement_key).is_none());
    assert_eq!(original.joint_for_key(&key).unwrap().node_index, 10);
}
