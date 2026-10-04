use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

#[test]
fn source_deletion_removes_root_and_subassets_without_touching_other_sources() {
    let source = AssetUri::parse("res://models/ship.glb").unwrap();
    let root_uuid = AssetUuid::from_stable_label("delete-root");
    let mesh_uuid = AssetUuid::from_stable_label("delete-mesh");
    let other_uuid = AssetUuid::from_stable_label("delete-other");
    let index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(root_uuid, source.clone(), AssetKind::Model, "root"),
        AssetRegistryEntry::new(
            mesh_uuid,
            AssetUri::parse("res://models/ship.glb#mesh").unwrap(),
            AssetKind::Model,
            "mesh",
        ),
        AssetRegistryEntry::new(
            other_uuid,
            AssetUri::parse("res://models/other.glb").unwrap(),
            AssetKind::Model,
            "other",
        ),
    ])
    .unwrap();

    let (candidate, removed) = index.prepare_source_deletion_generation(&source).unwrap();

    assert_eq!(removed, [root_uuid, mesh_uuid].into_iter().collect());
    assert!(candidate.entry_by_uuid(root_uuid).is_none());
    assert!(candidate.entry_by_uuid(mesh_uuid).is_none());
    assert!(candidate.entry_by_uuid(other_uuid).is_some());
}
