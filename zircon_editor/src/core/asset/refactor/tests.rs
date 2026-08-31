use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

use super::{AssetDeleteDisposition, AssetDeletePreflight};
use crate::core::asset::AssetSourceWritePolicy;

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn entry(label: &str, path: &str, dependencies: Vec<AssetUuid>) -> AssetRegistryEntry {
    AssetRegistryEntry::new(uuid(label), uri(path), AssetKind::Data, label)
        .with_dependencies(dependencies)
}

fn registry(entries: impl IntoIterator<Item = AssetRegistryEntry>) -> AssetRegistryIndex {
    AssetRegistryIndex::from_entries(entries).unwrap()
}

#[test]
fn delete_preflight_allows_an_unreferenced_writable_project_asset() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(preflight.disposition(), AssetDeleteDisposition::Allowed);
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
    assert!(preflight.referencers().is_empty());
}

#[test]
fn delete_preflight_rejects_a_missing_asset_without_path_fallback() {
    let registry = registry([entry("occupied", "res://data/reused.json", Vec::new())]);
    let missing_uuid = uuid("deleted-asset");

    let preflight = AssetDeletePreflight::evaluate(
        &registry,
        missing_uuid,
        AssetSourceWritePolicy::ProjectOnly,
    );

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::MissingAsset
    );
    assert!(preflight.target().is_none());
    assert!(preflight.referencers().is_empty());
}

#[test]
fn delete_preflight_rejects_non_project_sources_even_for_mutable_asset_types() {
    let target = entry(
        "package-target",
        "package://sample/data/target.json",
        Vec::new(),
    );
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::ReadOnlySource
    );
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
}

#[test]
fn delete_preflight_rejects_labeled_subassets_without_a_source_mutation_plan() {
    let target = entry("mesh-subasset", "res://models/ship.glb#mesh", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::UnsupportedSubasset
    );
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
}

#[test]
fn delete_preflight_blocks_and_projects_referencers_in_stable_registry_order() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let referencer_z = entry("referencer-z", "res://data/z.json", vec![target_uuid]);
    let referencer_a = entry("referencer-a", "res://data/a.json", vec![target_uuid]);
    let registry = registry([target, referencer_z, referencer_a]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::BlockedByReferencers
    );
    assert_eq!(
        preflight
            .referencers()
            .iter()
            .map(|referencer| referencer.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://data/a.json", "res://data/z.json"]
    );
}
