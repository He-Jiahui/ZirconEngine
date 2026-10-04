use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::{AssetRegistryEntry, AssetRegistryIndex};

#[test]
fn source_relocation_preserves_identity_and_retargets_external_referencers() {
    let root_uuid = AssetUuid::new();
    let subasset_uuid = AssetUuid::new();
    let external_uuid = AssetUuid::new();
    let source = AssetUri::parse("res://models/robot.glb").unwrap();
    let source_subasset = AssetUri::parse("res://models/robot.glb#mesh0").unwrap();
    let target = AssetUri::parse("res://actors/robot.glb").unwrap();
    let target_subasset = AssetUri::parse("res://actors/robot.glb#mesh0").unwrap();
    let external = AssetUri::parse("res://scenes/robot_scene.zscene").unwrap();
    let index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(root_uuid, source.clone(), AssetKind::Model, "robot"),
        AssetRegistryEntry::new(
            subasset_uuid,
            source_subasset.clone(),
            AssetKind::Model,
            "robot",
        ),
        AssetRegistryEntry::new(external_uuid, external, AssetKind::Scene, "scene")
            .with_dependencies(vec![root_uuid, subasset_uuid]),
    ])
    .expect("fixture registry should be valid");
    let mut relocated_meta = AssetMetaDocument::new(root_uuid, target.clone(), AssetKind::Model);
    relocated_meta.source_digest = "robot".to_owned();
    relocated_meta.entries = vec![
        AssetMetaEntry {
            uuid: root_uuid,
            url: target.clone(),
            asset_kind: AssetKind::Model,
            artifact_locator: None,
            dependencies: Vec::new(),
            tags: Default::default(),
        },
        AssetMetaEntry {
            uuid: subasset_uuid,
            url: target_subasset.clone(),
            asset_kind: AssetKind::Model,
            artifact_locator: None,
            dependencies: Vec::new(),
            tags: Default::default(),
        },
    ];

    let (candidate, affected) = index
        .prepare_source_relocation_generation(&source, &relocated_meta)
        .expect("source relocation should preserve the registered identities");

    assert!(candidate.entry_by_path(&source).is_none());
    assert!(candidate.entry_by_path(&source_subasset).is_none());
    assert_eq!(candidate.entry_by_path(&target).unwrap().uuid(), root_uuid);
    assert_eq!(
        candidate.entry_by_path(&target_subasset).unwrap().uuid(),
        subasset_uuid
    );
    assert_eq!(
        candidate.get_dependencies_by_uuid(external_uuid),
        vec![root_uuid, subasset_uuid]
    );
    assert_eq!(
        candidate.get_referencers_by_path(&target),
        vec![external_uuid]
    );
    assert_eq!(
        candidate.get_referencers_by_path(&target_subasset),
        vec![external_uuid]
    );
    assert!(affected.contains(&root_uuid));
    assert!(affected.contains(&subasset_uuid));
    assert!(affected.contains(&external_uuid));
}

#[test]
fn source_relocation_rejects_source_digest_drift() {
    let root_uuid = AssetUuid::new();
    let source = AssetUri::parse("res://models/original.glb").unwrap();
    let target = AssetUri::parse("res://models/relocated.glb").unwrap();
    let index = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        root_uuid,
        source.clone(),
        AssetKind::Model,
        "original-digest",
    )])
    .expect("fixture registry should be valid");
    let mut relocated_meta = AssetMetaDocument::new(root_uuid, target, AssetKind::Model);
    relocated_meta.source_digest = "changed-digest".to_owned();

    let error = index
        .prepare_source_relocation_generation(&source, &relocated_meta)
        .expect_err("relocation must not include source content changes");

    assert!(matches!(
        error,
        super::super::AssetRegistryError::SourceRelocationIdentityMismatch { reason, .. }
            if reason.contains("source digest")
    ));
}
