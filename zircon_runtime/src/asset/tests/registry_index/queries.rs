use std::collections::BTreeSet;

use crate::asset::registry::{AssetRegistryEntry, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::{AssetId, AssetKind, AssetUuid};

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
        .with_dependencies(vec![shader, texture])
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
}
