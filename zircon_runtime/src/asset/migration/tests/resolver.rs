use std::path::PathBuf;

use serde_json::json;
use zircon_runtime_interface::project::{
    migrate_retired_persisted_asset_reference_with, RelPath, RetiredAssetRefMigrationError,
};

use super::*;
use crate::asset::migration::{MigrationResolverIndex, MigrationSourceProjection};
use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

#[test]
fn retired_migration_never_downgrades_missing_labeled_subasset_to_parent() {
    let parent: AssetUuid = "c1111111-2222-4333-8444-555555555555".parse().unwrap();
    let root = PathBuf::from("E:/migration-resolver-test");
    let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        parent,
        AssetUri::parse("res://models/hero.glb").unwrap(),
        AssetKind::Model,
        "hero-digest",
    )])
    .unwrap();
    let sources = MigrationResolverIndex::build(
        [MigrationSourceProjection::new(
            RelPath::parse("assets").unwrap(),
            root.join("assets"),
            RelPath::parse("models/hero.glb").unwrap(),
            root.join("assets/models/hero.glb"),
        )],
        [],
    )
    .unwrap();
    let resolver = MigrationResolver::new(&registry, &sources);

    let error = migrate_retired_persisted_asset_reference_with(
        json!({
            "uuid": parent.to_string(),
            "url": "res://models/hero.glb#MissingMesh",
        }),
        |reference| resolver.resolve(reference),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        RetiredAssetRefMigrationError::Resolve(failure)
            if failure.kind == AssetMigrationIssueKind::DanglingReference
                && failure.message.contains("missing subasset MissingMesh")
    ));
}

#[test]
fn retired_migration_uses_a_matching_guid_to_repair_a_moved_source() {
    let guid: AssetUuid = "c3111111-2222-4333-8444-555555555555".parse().unwrap();
    let root = PathBuf::from("E:/migration-resolver-test");
    let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        guid,
        AssetUri::parse("res://models/hero.glb").unwrap(),
        AssetKind::Model,
        "hero-digest",
    )])
    .unwrap();
    let sources = MigrationResolverIndex::build(
        [MigrationSourceProjection::new(
            RelPath::parse("assets").unwrap(),
            root.join("assets"),
            RelPath::parse("models/hero.glb").unwrap(),
            root.join("assets/models/hero.glb"),
        )],
        [],
    )
    .unwrap();
    let resolver = MigrationResolver::new(&registry, &sources);

    let migrated = migrate_retired_persisted_asset_reference_with(
        json!({
            "uuid": guid.to_string(),
            "url": "res://legacy/hero.glb",
        }),
        |reference| resolver.resolve(reference),
    )
    .unwrap();

    assert_eq!(
        migrated,
        json!({
            "kind": "project",
            "guid": guid.to_string(),
            "path_hint": "assets/models/hero.glb",
            "sub": null,
        })
    );
}

#[test]
fn retired_migration_rejects_parent_guid_subasset_conflicts() {
    let parent: AssetUuid = "c4111111-2222-4333-8444-555555555555".parse().unwrap();
    let mesh: AssetUuid = "c5111111-2222-4333-8444-555555555555".parse().unwrap();
    let root = PathBuf::from("E:/migration-resolver-test");
    let registry = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            parent,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "hero-digest",
        ),
        AssetRegistryEntry::new(
            mesh,
            AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
            AssetKind::Mesh,
            "mesh-digest",
        ),
    ])
    .unwrap();
    let sources = MigrationResolverIndex::build(
        [MigrationSourceProjection::new(
            RelPath::parse("assets").unwrap(),
            root.join("assets"),
            RelPath::parse("models/hero.glb").unwrap(),
            root.join("assets/models/hero.glb"),
        )],
        [],
    )
    .unwrap();
    let resolver = MigrationResolver::new(&registry, &sources);

    for locator in ["res://models/hero.glb#Mesh0", "res://legacy/hero.glb#Mesh0"] {
        let error = migrate_retired_persisted_asset_reference_with(
            json!({
                "uuid": parent.to_string(),
                "url": locator,
            }),
            |reference| resolver.resolve(reference),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            RetiredAssetRefMigrationError::Resolve(failure)
                if failure.kind == AssetMigrationIssueKind::RegistryConflict
                    && failure.message == format!(
                        "asset reference guid {parent} and path hint assets/models/hero.glb resolve to different entries"
                    )
        ));
    }
}

#[test]
fn retired_migration_repairs_moved_subasset_hint_without_changing_identity() {
    let parent: AssetUuid = "c6111111-2222-4333-8444-555555555555".parse().unwrap();
    let mesh: AssetUuid = "c7111111-2222-4333-8444-555555555555".parse().unwrap();
    let root = PathBuf::from("E:/migration-resolver-test");
    let registry = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            parent,
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "hero-digest",
        ),
        AssetRegistryEntry::new(
            mesh,
            AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
            AssetKind::Mesh,
            "mesh-digest",
        ),
    ])
    .unwrap();
    let sources = MigrationResolverIndex::build(
        [MigrationSourceProjection::new(
            RelPath::parse("assets").unwrap(),
            root.join("assets"),
            RelPath::parse("models/hero.glb").unwrap(),
            root.join("assets/models/hero.glb"),
        )],
        [],
    )
    .unwrap();
    let resolver = MigrationResolver::new(&registry, &sources);

    let migrated = migrate_retired_persisted_asset_reference_with(
        json!({
            "uuid": mesh.to_string(),
            "url": "res://legacy/hero.glb#Mesh0",
        }),
        |reference| resolver.resolve(reference),
    )
    .unwrap();

    assert_eq!(
        migrated,
        json!({
            "kind": "project",
            "guid": mesh.to_string(),
            "path_hint": "assets/models/hero.glb",
            "sub": "Mesh0",
        })
    );
}
