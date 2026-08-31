use std::collections::BTreeSet;

use crate::asset::registry::{AssetRegistryEntry, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

fn entry(label: &str, path: &str, kind: AssetKind) -> AssetRegistryEntry {
    AssetRegistryEntry::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(path).expect("valid asset URI"),
        kind,
        label,
    )
}

#[test]
fn type_postings_preserve_canonical_query_order_and_composed_filters() {
    let index = AssetRegistryIndex::from_entries([
        entry("material-b", "res://materials/b.zmaterial", AssetKind::Material),
        entry("texture", "res://textures/a.png", AssetKind::Texture),
        entry("material-a", "res://materials/a.zmaterial", AssetKind::Material)
            .with_tags(BTreeSet::from(["surface".to_string()])),
    ])
    .expect("fixture entries have unique identity");

    let material_paths = index
        .get_assets_by_type(AssetKind::Material)
        .into_iter()
        .map(|entry| entry.path().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        material_paths,
        [
            "res://materials/a.zmaterial".to_string(),
            "res://materials/b.zmaterial".to_string(),
        ]
    );

    let filtered = index.get_assets(
        &AssetRegistryFilter::default()
            .with_type_marker(AssetKind::Material)
            .with_tag("surface"),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].path().to_string(), "res://materials/a.zmaterial");
}

#[test]
fn source_removal_retires_type_postings_and_empty_buckets() {
    let material_path = AssetUri::parse("res://materials/a.zmaterial").unwrap();
    let mut index = AssetRegistryIndex::from_entries([
        entry(
            "material",
            "res://materials/a.zmaterial",
            AssetKind::Material,
        ),
        entry("texture", "res://textures/a.png", AssetKind::Texture),
    ])
    .expect("fixture entries have unique identity");

    index.remove_source_path(&material_path);

    assert!(index.get_assets_by_type(AssetKind::Material).is_empty());
    assert!(!index.uuids_by_type.contains_key(&AssetKind::Material));
    assert_eq!(index.get_assets_by_type(AssetKind::Texture).len(), 1);
}
