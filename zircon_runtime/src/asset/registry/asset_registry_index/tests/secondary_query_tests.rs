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
fn secondary_postings_compose_tag_package_and_path_prefix_filters() {
    let index = AssetRegistryIndex::from_entries([
        entry(
            "hero-material",
            "package://game/materials/hero.zmat",
            AssetKind::Material,
        )
        .with_tags(BTreeSet::from(["hero".to_string(), "surface".to_string()])),
        entry(
            "hero-material-preview",
            "package://game/materials/hero.zmat#preview",
            AssetKind::Material,
        )
        .with_tags(BTreeSet::from(["hero".to_string()])),
        entry(
            "enemy-material",
            "package://game/materials/enemy.zmat",
            AssetKind::Material,
        )
        .with_tags(BTreeSet::from(["enemy".to_string()])),
        entry(
            "hero-texture",
            "package://other/materials/hero.png",
            AssetKind::Texture,
        )
        .with_tags(BTreeSet::from(["hero".to_string()])),
    ])
    .expect("fixture entries have unique identity");

    let filtered = index.get_assets(
        &AssetRegistryFilter::default()
            .with_tag("hero")
            .with_package("game")
            .with_path_prefix("game/materials/hero"),
    );
    let paths = filtered
        .into_iter()
        .map(|entry| entry.path().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        [
            "package://game/materials/hero.zmat".to_string(),
            "package://game/materials/hero.zmat#preview".to_string(),
        ]
    );
}

#[test]
fn direct_posting_candidate_still_applies_path_filter_without_union() {
    let index = AssetRegistryIndex::from_entries([entry(
        "hero-outside-prefix",
        "package://game/materials/other.zmat",
        AssetKind::Material,
    )
    .with_tags(BTreeSet::from(["hero".to_string()]))])
    .expect("fixture entry has unique identity");

    assert!(index
        .get_assets(
            &AssetRegistryFilter::default()
                .with_tag("hero")
                .with_path_prefix("game/materials/hero"),
        )
        .is_empty());
}

#[test]
fn secondary_postings_are_retired_with_source_entries() {
    let removed_path = AssetUri::parse("package://game/materials/hero.zmat").unwrap();
    let survivor_uuid = AssetUuid::from_stable_label("survivor");
    let removed_uuid = AssetUuid::from_stable_label("removed");
    let mut index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            removed_uuid,
            removed_path.clone(),
            AssetKind::Material,
            "removed",
        )
        .with_tags(BTreeSet::from(["hero".to_string()])),
        AssetRegistryEntry::new(
            AssetUuid::from_stable_label("removed-preview"),
            AssetUri::parse("package://game/materials/hero.zmat#preview").unwrap(),
            AssetKind::Material,
            "removed-preview",
        )
        .with_tags(BTreeSet::from(["hero".to_string()])),
        AssetRegistryEntry::new(
            survivor_uuid,
            AssetUri::parse("res://materials/survivor.zmat").unwrap(),
            AssetKind::Material,
            "survivor",
        )
        .with_tags(BTreeSet::from(["survivor".to_string()])),
    ])
    .expect("fixture entries have unique identity");

    index.remove_source_path(&removed_path);

    assert!(index
        .get_assets(&AssetRegistryFilter::default().with_tag("hero"))
        .is_empty());
    assert!(!index.uuids_by_tag.contains_key("hero"));
    assert!(!index.uuids_by_package.contains_key("game"));
    assert!(!index
        .uuids_by_path_prefix
        .contains_key("game/materials/hero.zmat"));
    assert_eq!(index.get_assets_by_type(AssetKind::Material).len(), 1);
    assert_eq!(
        index.entry_by_uuid(survivor_uuid).map(|entry| entry.uuid()),
        Some(survivor_uuid)
    );
}

#[test]
fn referencer_query_keeps_deterministic_uuid_order() {
    let dependency = AssetUuid::from_stable_label("referencer-dependency");
    let first = AssetUuid::from_stable_label("referencer-first");
    let second = AssetUuid::from_stable_label("referencer-second");
    let index = AssetRegistryIndex::from_entries([
        entry(
            "referencer-dependency",
            "res://data/dependency.asset",
            AssetKind::Data,
        ),
        AssetRegistryEntry::new(
            first,
            AssetUri::parse("res://data/first.asset").unwrap(),
            AssetKind::Data,
            "first",
        )
        .with_dependencies(vec![dependency]),
        AssetRegistryEntry::new(
            second,
            AssetUri::parse("res://data/second.asset").unwrap(),
            AssetKind::Data,
            "second",
        )
        .with_dependencies(vec![dependency]),
    ])
    .expect("fixture entries have unique identity");

    let mut expected = vec![first, second];
    expected.sort_unstable_by(|left, right| left.binary_key().cmp(right.binary_key()));
    let mut display_order = vec![first, second];
    display_order.sort_by_key(ToString::to_string);
    assert_eq!(expected, display_order);
    assert_eq!(index.get_referencers_by_uuid(dependency), expected);
}
