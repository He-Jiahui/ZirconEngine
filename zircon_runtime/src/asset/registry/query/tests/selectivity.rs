use std::collections::BTreeSet;

use super::{AssetRegistryEntry, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid};

fn entry(label: &str, path: &str, kind: AssetKind, tags: &[&str]) -> AssetRegistryEntry {
    AssetRegistryEntry::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(path).expect("valid fixture URI"),
        kind,
        label,
    )
    .with_tags(
        tags.iter()
            .map(|tag| (*tag).to_string())
            .collect::<BTreeSet<_>>(),
    )
}

fn fixture() -> AssetRegistryIndex {
    let mut entries = (0..128)
        .map(|i| {
            entry(
                &format!("broad-{i}"),
                &format!("res://broad/{i:03}.asset"),
                AssetKind::Material,
                &["common"],
            )
        })
        .collect::<Vec<_>>();
    entries.extend([
        entry(
            "z",
            "package://game/target/z.asset",
            AssetKind::Material,
            &["rare", "common"],
        ),
        entry(
            "a",
            "package://game/target/a.asset",
            AssetKind::Material,
            &["rare", "common"],
        ),
        entry(
            "preview",
            "package://game/target/a.asset#preview",
            AssetKind::Material,
            &["rare", "common"],
        ),
        entry(
            "other-package",
            "package://other/target/a.asset",
            AssetKind::Material,
            &["rare", "common"],
        ),
        entry(
            "other-prefix",
            "package://game/outside/a.asset",
            AssetKind::Material,
            &["rare", "common"],
        ),
        entry(
            "other-tag",
            "package://game/target/no-tag.asset",
            AssetKind::Material,
            &["common"],
        ),
        entry(
            "other-type",
            "package://game/target/texture.asset",
            AssetKind::Texture,
            &["rare", "common"],
        ),
    ]);
    AssetRegistryIndex::from_entries(entries).expect("fixture identities are unique")
}

fn scan(index: &AssetRegistryIndex, filter: &AssetRegistryFilter) -> Vec<AssetUuid> {
    let mut entries = index
        .entries_iter()
        .filter(|entry| {
            filter
                .type_marker
                .is_none_or(|kind| entry.type_marker() == kind)
                && filter
                    .required_tags
                    .iter()
                    .all(|tag| entry.tags().contains(tag))
                && filter
                    .package_id
                    .as_deref()
                    .is_none_or(|package| entry.path().package_id() == Some(package))
                && filter
                    .path_prefix
                    .as_deref()
                    .is_none_or(|prefix| entry.path().path().starts_with(prefix))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    entries.into_iter().map(AssetRegistryEntry::uuid).collect()
}

fn assert_matches_scan(index: &AssetRegistryIndex, filter: &AssetRegistryFilter) {
    let actual = index
        .get_assets(filter)
        .into_iter()
        .map(AssetRegistryEntry::uuid)
        .collect::<Vec<_>>();
    assert_eq!(actual, scan(index, filter), "filter: {filter:?}");
}

#[test]
fn runtime51_selectivity_broad_type_with_small_tag_preserves_canonical_results() {
    let index = fixture();
    let filter = AssetRegistryFilter::default()
        .with_type_marker(AssetKind::Material)
        .with_tag("rare");
    assert_eq!(index.get_assets(&filter).len(), 5);
    assert_matches_scan(&index, &filter);
}

#[test]
fn runtime51_selectivity_broad_type_with_small_package_preserves_canonical_results() {
    let index = fixture();
    let filter = AssetRegistryFilter::default()
        .with_type_marker(AssetKind::Material)
        .with_package("game");
    assert_eq!(index.get_assets(&filter).len(), 5);
    assert_matches_scan(&index, &filter);
}

#[test]
fn runtime51_selectivity_intersections_missing_postings_and_type_prefix_match_scan() {
    let index = fixture();
    let typed = AssetRegistryFilter::default().with_type_marker(AssetKind::Material);
    let intersected = typed
        .clone()
        .with_tag("common")
        .with_tag("rare")
        .with_package("game")
        .with_path_prefix("game/target/a.asset");
    assert_eq!(index.get_assets(&intersected).len(), 2);
    for filter in [
        intersected,
        typed.clone(),
        typed.clone().with_path_prefix("game/target/a.asset"),
        typed.clone().with_tag("common"),
        AssetRegistryFilter::default()
            .with_type_marker(AssetKind::Texture)
            .with_tag("common"),
        AssetRegistryFilter::default()
            .with_type_marker(AssetKind::Texture)
            .with_package("other"),
        typed.clone().with_tag("missing"),
        typed.clone().with_package("missing"),
        typed.clone().with_tag("rare").with_package("missing"),
        typed.clone().with_tag("rare").with_path_prefix("missing"),
        AssetRegistryFilter::default()
            .with_type_marker(AssetKind::Data)
            .with_tag("rare"),
        AssetRegistryFilter::default()
            .with_tag("rare")
            .with_package("game"),
        AssetRegistryFilter::default().with_path_prefix("game/target/a.asset"),
        AssetRegistryFilter::default(),
    ] {
        assert_matches_scan(&index, &filter);
    }
}

#[test]
fn runtime51_selectivity_composed_offline_query_remains_correct_after_source_removal() {
    let mut index = fixture();
    index.remove_source_path(&AssetUri::parse("package://game/target/a.asset").unwrap());
    let filter = AssetRegistryFilter::default()
        .with_type_marker(AssetKind::Material)
        .with_tag("rare")
        .with_package("game")
        .with_path_prefix("game/target/a.asset");
    assert!(index.get_assets(&filter).is_empty());
    assert_matches_scan(&index, &filter);
}
