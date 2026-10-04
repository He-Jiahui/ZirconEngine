use std::collections::HashSet;

use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::{AssetRegistryEntry, AssetRegistryIndex};

#[test]
fn persisted_entries_restore_dependency_path_and_referencer_indexes() {
    let dependency_uuid = AssetUuid::new();
    let owner_uuid = AssetUuid::new();
    let dependency_path = AssetUri::parse("res://data/dependency.json").unwrap();
    let owner_path = AssetUri::parse("res://data/owner.json").unwrap();
    let index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            dependency_uuid,
            dependency_path.clone(),
            AssetKind::Data,
            "dependency",
        ),
        AssetRegistryEntry::new(owner_uuid, owner_path, AssetKind::Data, "owner")
            .with_dependencies(vec![dependency_uuid]),
    ])
    .unwrap();

    assert_eq!(
        index.get_referencers_by_path(&dependency_path),
        vec![owner_uuid]
    );
}

#[test]
fn overlapping_dependency_path_sources_resolve_to_one_uuid_edge() {
    let dependency_uuid = AssetUuid::new();
    let owner_uuid = AssetUuid::new();
    let dependency_path = AssetUri::parse("res://data/dependency.json").unwrap();
    let mut index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            dependency_uuid,
            dependency_path.clone(),
            AssetKind::Data,
            "dependency",
        ),
        AssetRegistryEntry::new(
            owner_uuid,
            AssetUri::parse("res://data/owner.json").unwrap(),
            AssetKind::Data,
            "owner",
        ),
    ])
    .unwrap();
    index.replace_dependency_paths(owner_uuid, vec![dependency_path.clone(), dependency_path]);

    index.refresh_dependency_owners(&HashSet::from([owner_uuid]));

    assert_eq!(
        index.entry_by_uuid(owner_uuid).unwrap().dependencies(),
        &[dependency_uuid]
    );
}
