use std::collections::BTreeSet;

use crate::asset::registry::{AssetRegistryEntry, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::{AssetId, AssetKind, AssetReference, AssetUuid};

use super::uri;

#[test]
fn registry_exposes_all_six_offline_query_signatures() {
    let shader = AssetUuid::new();
    let texture = AssetUuid::new();
    let material = AssetUuid::new();
    let scene = AssetUuid::new();
    let index = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            shader,
            uri("res://shaders/pbr.zshader"),
            AssetKind::Shader,
            "s",
        ),
        AssetRegistryEntry::new(
            texture,
            uri("res://textures/hero.png"),
            AssetKind::Texture,
            "t",
        )
        .with_tags(BTreeSet::from(["hero".to_string()])),
        AssetRegistryEntry::new(
            material,
            uri("res://materials/hero.zmaterial"),
            AssetKind::Material,
            "m",
        )
        .with_dependencies(vec![shader, texture, shader])
        .with_tags(BTreeSet::from(["hero".to_string(), "surface".to_string()])),
        AssetRegistryEntry::new(
            scene,
            uri("package://game/scenes/main.scene.toml"),
            AssetKind::Scene,
            "w",
        )
        .with_dependencies(vec![material]),
    ])
    .unwrap();

    assert_eq!(
        index.get_assets_by_type(AssetKind::Material)[0].uuid(),
        material
    );
    assert_eq!(
        index
            .get_assets(&AssetRegistryFilter::default().with_tag("hero"))
            .len(),
        2
    );
    assert_eq!(
        index.get_dependencies_by_uuid(material),
        vec![shader, texture]
    );
    assert_eq!(
        index.get_dependencies_by_path(&uri("res://materials/hero.zmaterial")),
        vec![shader, texture]
    );
    assert_eq!(index.get_referencers_by_uuid(material), vec![scene]);
    assert_eq!(
        index.get_referencers_by_path(&uri("res://materials/hero.zmaterial")),
        vec![scene]
    );
    assert_eq!(
        index.resolve_asset_id_by_uuid(material),
        Ok(AssetId::from_asset_uuid(material))
    );
    assert_eq!(
        index.resolve_reference_by_asset_id(AssetId::from_asset_uuid(material)),
        Ok(AssetReference::new(
            material,
            uri("res://materials/hero.zmaterial")
        ))
    );
}

#[test]
fn registry_hot_identity_and_reverse_queries_use_derived_indexes() {
    let index_source = include_str!("../../registry/asset_registry_index.rs");
    let query_source = include_str!("../../registry/query.rs");

    assert!(index_source.contains("uuid_by_asset_id"));
    assert!(index_source.contains("referencers_by_uuid"));
    assert!(index_source.contains("entry_uuids_by_source"));
    assert!(index_source.contains("dependency_paths_by_uuid"));
    assert!(index_source.contains("referencers_by_path"));
    assert!(
        !query_source
            .contains(".values()\n            .filter(|entry| entry.dependencies().contains")
    );
    assert!(
        !query_source.contains(".values()\n            .find(|entry| AssetId::from_asset_uuid")
    );
    assert!(!index_source.contains(".values()\n            .filter(|entry| same_source_path"));
    let targeted_source = include_str!("../../registry/targeted.rs");
    assert!(!targeted_source.contains("entries_by_uuid\n            .values()"));
}

#[test]
fn registry_dependency_dedup_is_linear_and_preserves_first_seen_order() {
    let source = include_str!("../../registry/asset_registry_entry.rs");

    assert!(source.contains("HashSet::with_capacity(dependencies.len())"));
    assert!(!source.contains("if !unique.contains(&dependency)"));
}

#[test]
fn registry_dependency_refresh_borrows_scanned_meta_edges() {
    let source = include_str!("../../registry/rebuild.rs");

    assert!(source.contains("let mut dependency_paths: HashMap<AssetUuid, &[AssetUri]>"));
    assert!(!source.contains(
        "fn dependency_paths(meta: &AssetMetaDocument) -> Vec<(AssetUuid, Vec<AssetUri>)>"
    ));
}
