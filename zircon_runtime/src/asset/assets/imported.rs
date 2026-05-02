use serde::{Deserialize, Serialize};

use super::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, DataAsset, FontAsset, MaterialAsset, MaterialGraphAsset,
    ModelAsset, NavMeshAsset, NavigationSettingsAsset, PhysicsMaterialAsset, PrefabAsset,
    SceneAsset, ShaderAsset, SoundAsset, TerrainAsset, TerrainLayerStackAsset, TextureAsset,
    TileMapAsset, TileSetAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};
use crate::asset::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportedAsset {
    Data(DataAsset),
    Texture(TextureAsset),
    Shader(ShaderAsset),
    Material(MaterialAsset),
    MaterialGraph(MaterialGraphAsset),
    Sound(SoundAsset),
    Font(FontAsset),
    PhysicsMaterial(PhysicsMaterialAsset),
    NavMesh(NavMeshAsset),
    NavigationSettings(NavigationSettingsAsset),
    Terrain(TerrainAsset),
    TerrainLayerStack(TerrainLayerStackAsset),
    TileSet(TileSetAsset),
    TileMap(TileMapAsset),
    Prefab(PrefabAsset),
    Scene(SceneAsset),
    Model(ModelAsset),
    AnimationSkeleton(AnimationSkeletonAsset),
    AnimationClip(AnimationClipAsset),
    AnimationSequence(AnimationSequenceAsset),
    AnimationGraph(AnimationGraphAsset),
    AnimationStateMachine(AnimationStateMachineAsset),
    UiLayout(UiLayoutAsset),
    UiWidget(UiWidgetAsset),
    UiStyle(UiStyleAsset),
}

impl ImportedAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        match self {
            Self::AnimationClip(asset) => asset.direct_references(),
            Self::AnimationGraph(asset) => asset.direct_references(),
            Self::AnimationStateMachine(asset) => asset.direct_references(),
            Self::MaterialGraph(asset) => asset.direct_references(),
            Self::Scene(asset) => asset.direct_references(),
            Self::Terrain(asset) => asset.direct_references(),
            Self::TerrainLayerStack(asset) => asset.direct_references(),
            Self::TileSet(asset) => asset.direct_references(),
            Self::TileMap(asset) => asset.direct_references(),
            Self::Prefab(asset) => asset.direct_references(),
            _ => Vec::new(),
        }
    }
}

pub fn asset_kind_for_imported_asset(imported: &ImportedAsset) -> crate::asset::AssetKind {
    match imported {
        ImportedAsset::Data(_) => crate::asset::AssetKind::Data,
        ImportedAsset::Texture(_) => crate::asset::AssetKind::Texture,
        ImportedAsset::Shader(_) => crate::asset::AssetKind::Shader,
        ImportedAsset::Material(_) => crate::asset::AssetKind::Material,
        ImportedAsset::MaterialGraph(_) => crate::asset::AssetKind::MaterialGraph,
        ImportedAsset::Sound(_) => crate::asset::AssetKind::Sound,
        ImportedAsset::Font(_) => crate::asset::AssetKind::Font,
        ImportedAsset::PhysicsMaterial(_) => crate::asset::AssetKind::PhysicsMaterial,
        ImportedAsset::NavMesh(_) => crate::asset::AssetKind::NavMesh,
        ImportedAsset::NavigationSettings(_) => crate::asset::AssetKind::NavigationSettings,
        ImportedAsset::Terrain(_) => crate::asset::AssetKind::Terrain,
        ImportedAsset::TerrainLayerStack(_) => crate::asset::AssetKind::TerrainLayerStack,
        ImportedAsset::TileSet(_) => crate::asset::AssetKind::TileSet,
        ImportedAsset::TileMap(_) => crate::asset::AssetKind::TileMap,
        ImportedAsset::Prefab(_) => crate::asset::AssetKind::Prefab,
        ImportedAsset::Scene(_) => crate::asset::AssetKind::Scene,
        ImportedAsset::Model(_) => crate::asset::AssetKind::Model,
        ImportedAsset::AnimationSkeleton(_) => crate::asset::AssetKind::AnimationSkeleton,
        ImportedAsset::AnimationClip(_) => crate::asset::AssetKind::AnimationClip,
        ImportedAsset::AnimationSequence(_) => crate::asset::AssetKind::AnimationSequence,
        ImportedAsset::AnimationGraph(_) => crate::asset::AssetKind::AnimationGraph,
        ImportedAsset::AnimationStateMachine(_) => crate::asset::AssetKind::AnimationStateMachine,
        ImportedAsset::UiLayout(_) => crate::asset::AssetKind::UiLayout,
        ImportedAsset::UiWidget(_) => crate::asset::AssetKind::UiWidget,
        ImportedAsset::UiStyle(_) => crate::asset::AssetKind::UiStyle,
    }
}
