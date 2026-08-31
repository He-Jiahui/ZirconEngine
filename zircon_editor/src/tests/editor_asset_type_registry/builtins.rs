use std::collections::BTreeSet;

use crate::core::asset::{AssetTypeId, AssetTypeRegistry, ThumbnailProviderDescriptor};
use crate::core::commands::EditorCommandRegistry;
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

#[test]
fn builtin_animation_assets_have_production_toolkit_routes_and_open_operations() {
    let asset_types = AssetTypeRegistry::with_builtins().unwrap();
    let commands = EditorCommandRegistry::default_workbench();

    for (kind, view_id, operation) in [
        (
            ResourceKind::AnimationSequence,
            "editor.animation_sequence",
            "timeline_sequence.authoring.open",
        ),
        (
            ResourceKind::AnimationGraph,
            "editor.animation_graph",
            "animation_graph.authoring.open_graph",
        ),
        (
            ResourceKind::AnimationStateMachine,
            "editor.animation_graph",
            "animation_graph.authoring.open_state_machine",
        ),
    ] {
        let asset_type = AssetTypeId::from_resource_kind(kind);
        let toolkit = asset_types
            .get(&asset_type)
            .and_then(|definition| definition.toolkit())
            .expect("built-in animation asset type should declare a toolkit");

        assert_eq!(toolkit.view_id(), view_id);
        assert_eq!(toolkit.open_operation().as_str(), operation);
        assert!(
            toolkit.required_capabilities().is_empty(),
            "production animation toolkit must not depend on a test-only extension capability"
        );
        assert!(
            commands.command(operation).is_some(),
            "built-in animation toolkit operation must be registered"
        );
    }
}
