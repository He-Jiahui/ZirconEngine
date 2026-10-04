use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::{AssetRegistryEntry, AssetRegistryIndex};

#[test]
fn bulk_build_streams_dependency_path_projection_without_changing_edges() {
    let dependency_uuid = AssetUuid::from_stable_label("bulk-build-dependency");
    let owner_uuid = AssetUuid::from_stable_label("bulk-build-owner");
    let dependency_path = AssetUri::parse("res://bulk/dependency.asset").unwrap();
    let owner_path = AssetUri::parse("res://bulk/owner.asset").unwrap();

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
    .expect("fixture entries have unique identity");

    assert_eq!(
        index.get_dependencies_by_uuid(owner_uuid),
        vec![dependency_uuid]
    );
    assert_eq!(
        index.get_referencers_by_path(&dependency_path),
        vec![owner_uuid]
    );
}

#[test]
fn bulk_build_ignores_unresolved_dependency_paths_until_resolution() {
    let owner_uuid = AssetUuid::from_stable_label("bulk-build-unresolved-owner");
    let missing_uuid = AssetUuid::from_stable_label("bulk-build-missing-dependency");
    let owner_path = AssetUri::parse("res://bulk/unresolved-owner.asset").unwrap();

    let index = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        owner_uuid,
        owner_path,
        AssetKind::Data,
        "owner",
    )
    .with_dependencies(vec![missing_uuid])])
    .expect("fixture entry has unique identity");

    assert_eq!(
        index.get_dependencies_by_uuid(owner_uuid),
        vec![missing_uuid]
    );
    assert_eq!(
        index.get_referencers_by_uuid(missing_uuid),
        vec![owner_uuid]
    );
    assert!(index.referencers_by_path.is_empty());
}
