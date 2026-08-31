use crate::asset::mutation::{AssetMutationDeleteDisposition, AssetMutationDeletePreflight};
use crate::asset::mutation::{
    AssetMutationRelocationDisposition, AssetMutationRelocationPreflight,
};
use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("fixture uri is valid")
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn entry(label: &str, path: &str, dependencies: Vec<AssetUuid>) -> AssetRegistryEntry {
    AssetRegistryEntry::new(uuid(label), uri(path), AssetKind::Data, label)
        .with_dependencies(dependencies)
}

fn registry(entries: impl IntoIterator<Item = AssetRegistryEntry>) -> AssetRegistryIndex {
    AssetRegistryIndex::from_entries(entries).expect("fixture registry is valid")
}

#[test]
fn delete_preflight_allows_an_unreferenced_root_asset() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight = AssetMutationDeletePreflight::evaluate(&registry, target_uuid);

    assert_eq!(
        preflight.disposition(),
        AssetMutationDeleteDisposition::Ready
    );
    assert_eq!(
        preflight.target().map(|asset| asset.uuid()),
        Some(target_uuid)
    );
    assert!(preflight.referencers().is_empty());
}

#[test]
fn delete_preflight_rejects_labeled_subassets() {
    let target = entry("mesh", "res://models/ship.glb#mesh", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight = AssetMutationDeletePreflight::evaluate(&registry, target_uuid);

    assert_eq!(
        preflight.disposition(),
        AssetMutationDeleteDisposition::UnsupportedSubasset
    );
}

#[test]
fn delete_preflight_blocks_direct_referencers_in_canonical_locator_order() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([
        target,
        entry("referencer-z", "res://data/z.json", vec![target_uuid]),
        entry("referencer-a", "res://data/a.json", vec![target_uuid]),
    ]);

    let preflight = AssetMutationDeletePreflight::evaluate(&registry, target_uuid);

    assert_eq!(
        preflight.disposition(),
        AssetMutationDeleteDisposition::BlockedByReferencers
    );
    assert_eq!(
        preflight
            .referencers()
            .iter()
            .map(|asset| asset.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://data/a.json", "res://data/z.json"]
    );
}

#[test]
fn delete_preflight_checks_the_whole_source_and_ignores_internal_companion_edges() {
    let target = entry("target", "res://models/ship.glb", Vec::new());
    let target_uuid = target.uuid();
    let subasset = entry("mesh", "res://models/ship.glb#mesh", vec![target_uuid]);
    let subasset_uuid = subasset.uuid();
    let registry = registry([
        target,
        subasset,
        entry("external", "res://scenes/ship.zscene", vec![subasset_uuid]),
    ]);

    let preflight = AssetMutationDeletePreflight::evaluate(&registry, target_uuid);

    assert_eq!(
        preflight.disposition(),
        AssetMutationDeleteDisposition::BlockedByReferencers
    );
    assert_eq!(
        preflight
            .referencers()
            .iter()
            .map(|asset| asset.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://scenes/ship.zscene"]
    );
}

#[test]
fn relocation_preflight_collects_all_source_companions_and_external_referencers() {
    let source = entry("source", "res://data/source.json", Vec::new());
    let source_uuid = source.uuid();
    let subasset = entry(
        "source-sub",
        "res://data/source.json#payload",
        vec![source_uuid],
    );
    let registry = registry([
        source,
        subasset,
        entry("outside", "res://data/outside.json", vec![source_uuid]),
    ]);

    let preflight = AssetMutationRelocationPreflight::evaluate(
        &registry,
        source_uuid,
        uri("res://data/moved.json"),
    );

    assert_eq!(
        preflight.disposition(),
        AssetMutationRelocationDisposition::Ready
    );
    assert_eq!(
        preflight
            .companions()
            .iter()
            .map(|asset| asset.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://data/source.json", "res://data/source.json#payload"]
    );
    assert_eq!(
        preflight
            .referencer_closure()
            .iter()
            .map(|asset| asset.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://data/outside.json"]
    );
}

#[test]
fn relocation_preflight_rejects_an_occupied_target_without_rebinding_identity() {
    let source = entry("source", "res://data/source.json", Vec::new());
    let source_uuid = source.uuid();
    let registry = registry([
        source,
        entry("occupied", "res://data/occupied.json", Vec::new()),
    ]);

    let preflight = AssetMutationRelocationPreflight::evaluate(
        &registry,
        source_uuid,
        uri("res://data/occupied.json"),
    );

    assert_eq!(
        preflight.disposition(),
        AssetMutationRelocationDisposition::TargetOccupied
    );
    assert_eq!(
        preflight.target_occupant().map(|asset| asset.uuid()),
        Some(uuid("occupied"))
    );
}
