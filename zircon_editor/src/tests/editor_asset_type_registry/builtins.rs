use std::collections::BTreeSet;

use crate::core::asset::{AssetTypeId, AssetTypeRegistry, ThumbnailProviderDescriptor};
use zircon_runtime_interface::resource::ResourceKind;

const RESOURCE_KINDS: [ResourceKind; 26] = [
    ResourceKind::Data,
    ResourceKind::Model,
    ResourceKind::Mesh,
    ResourceKind::Material,
    ResourceKind::MaterialGraph,
    ResourceKind::Texture,
    ResourceKind::Shader,
    ResourceKind::Scene,
    ResourceKind::Sound,
    ResourceKind::Font,
    ResourceKind::PhysicsMaterial,
    ResourceKind::NavMesh,
    ResourceKind::NavigationSettings,
    ResourceKind::Terrain,
    ResourceKind::TerrainLayerStack,
    ResourceKind::TileSet,
    ResourceKind::TileMap,
    ResourceKind::Prefab,
    ResourceKind::AnimationSkeleton,
    ResourceKind::AnimationClip,
    ResourceKind::AnimationSequence,
    ResourceKind::AnimationGraph,
    ResourceKind::AnimationStateMachine,
    ResourceKind::UiLayout,
    ResourceKind::UiWidget,
    ResourceKind::UiStyle,
];

#[test]
fn every_runtime_resource_kind_has_one_unique_builtin_asset_type() {
    let registry = AssetTypeRegistry::with_builtins().unwrap();
    assert_eq!(registry.len(), RESOURCE_KINDS.len());

    let mut ids = BTreeSet::new();
    for kind in RESOURCE_KINDS {
        let id = AssetTypeId::from_resource_kind(kind);
        assert!(ids.insert(id.clone()), "duplicate builtin id {id}");
        let definition = registry.get(&id).expect("builtin asset type definition");
        assert_eq!(definition.runtime_kind(), Some(kind));
        assert!(!definition.presentation().display_name().is_empty());
        assert!(!definition.presentation().badge().is_empty());
        assert!(!definition.presentation().icon_name().is_empty());
        assert!(!definition.presentation().color_token().is_empty());
    }
}

#[test]
fn texture_builtin_declares_source_image_preview_without_ui_kind_dispatch() {
    let registry = AssetTypeRegistry::with_builtins().unwrap();
    let texture = AssetTypeId::from_resource_kind(ResourceKind::Texture);
    assert_eq!(
        registry.get(&texture).unwrap().thumbnail(),
        &ThumbnailProviderDescriptor::SourceImage
    );
}

#[test]
fn builtin_ids_match_first_party_plugin_authoring_keys() {
    assert_eq!(
        AssetTypeId::from_resource_kind(ResourceKind::Model).as_str(),
        "model"
    );
    assert_eq!(
        AssetTypeId::from_resource_kind(ResourceKind::MaterialGraph).as_str(),
        "material.graph"
    );
    assert_eq!(
        AssetTypeId::from_resource_kind(ResourceKind::AnimationStateMachine).as_str(),
        "animation.state_machine"
    );
    assert_eq!(
        AssetTypeId::from_resource_kind(ResourceKind::TileMap).as_str(),
        "tilemap_2d.tilemap"
    );
}
