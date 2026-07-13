use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetImportError, AssetUri, DataAsset, ImportedAsset, MaterialGraphAsset, ModelAsset,
    PhysicsMaterialAsset, PrefabAsset, SoundAsset, TerrainAsset, TerrainLayerStackAsset,
    TextureAsset, TexturePayload, TileMapAsset, TileSetAsset, UiIconAsset, UiThemeAsset,
};
use crate::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};
use crate::core::framework::navigation::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::framework::scene::physics::PhysicsMaterialMetadata;

mod font;
mod json_value;
mod material_shader;
mod mesh;
mod scene;
mod toml_value;
mod ui;

use font::ArtifactCacheFontAsset;
use json_value::ArtifactCacheJsonValue;
use material_shader::{ArtifactCacheMaterialAsset, ArtifactCacheShaderAsset};
use mesh::ArtifactCacheMeshAsset;
use scene::ArtifactCacheSceneAsset;
use ui::{ArtifactCacheUiAssetDocument, ArtifactCacheUiV2AssetDocument};

/// Bincode cache wire type. It keeps authoring-friendly serde shapes such as
/// TOML values and flattened fields out of runtime library artifacts.
/// Every field must serialize, even when optional, because bincode is not a
/// self-describing map format and skipped fields shift the remaining bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum ArtifactCacheAsset {
    Data(ArtifactCacheDataAsset),
    Texture(ArtifactCacheTextureAsset),
    Shader(ArtifactCacheShaderAsset),
    Material(ArtifactCacheMaterialAsset),
    MaterialGraph(MaterialGraphAsset),
    Sound(SoundAsset),
    Font(ArtifactCacheFontAsset),
    PhysicsMaterial(ArtifactCachePhysicsMaterialAsset),
    NavMesh(NavMeshAsset),
    NavigationSettings(NavigationSettingsAsset),
    Terrain(TerrainAsset),
    TerrainLayerStack(TerrainLayerStackAsset),
    TileSet(TileSetAsset),
    TileMap(TileMapAsset),
    Prefab(ArtifactCachePrefabAsset),
    Scene(ArtifactCacheSceneAsset),
    Model(ModelAsset),
    Mesh(ArtifactCacheMeshAsset),
    AnimationSkeleton(AnimationSkeletonAsset),
    AnimationClip(AnimationClipAsset),
    AnimationSequence(AnimationSequenceAsset),
    AnimationGraph(AnimationGraphAsset),
    AnimationStateMachine(AnimationStateMachineAsset),
    UiLayout(ArtifactCacheUiAssetDocument),
    UiWidget(ArtifactCacheUiAssetDocument),
    UiStyle(ArtifactCacheUiAssetDocument),
    UiTheme(UiThemeAsset),
    UiIcon(UiIconAsset),
    UiV2View(ArtifactCacheUiV2AssetDocument),
    UiV2Component(ArtifactCacheUiV2AssetDocument),
    UiV2Style(ArtifactCacheUiV2AssetDocument),
}

impl ArtifactCacheAsset {
    pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, AssetImportError> {
        Ok(match asset {
            ImportedAsset::Data(asset) => Self::Data(ArtifactCacheDataAsset::from(asset)),
            ImportedAsset::Texture(asset) => Self::Texture(ArtifactCacheTextureAsset::from(asset)),
            ImportedAsset::Shader(asset) => Self::Shader(ArtifactCacheShaderAsset::from(asset)),
            ImportedAsset::Material(asset) => {
                Self::Material(ArtifactCacheMaterialAsset::from(asset))
            }
            ImportedAsset::MaterialGraph(asset) => Self::MaterialGraph(asset.clone()),
            ImportedAsset::Sound(asset) => Self::Sound(asset.clone()),
            ImportedAsset::Font(asset) => Self::Font(ArtifactCacheFontAsset::from(asset)),
            ImportedAsset::PhysicsMaterial(asset) => {
                Self::PhysicsMaterial(ArtifactCachePhysicsMaterialAsset::from(asset))
            }
            ImportedAsset::NavMesh(asset) => Self::NavMesh(asset.clone()),
            ImportedAsset::NavigationSettings(asset) => Self::NavigationSettings(asset.clone()),
            ImportedAsset::Terrain(asset) => Self::Terrain(asset.clone()),
            ImportedAsset::TerrainLayerStack(asset) => Self::TerrainLayerStack(asset.clone()),
            ImportedAsset::TileSet(asset) => Self::TileSet(asset.clone()),
            ImportedAsset::TileMap(asset) => Self::TileMap(asset.clone()),
            ImportedAsset::Prefab(asset) => Self::Prefab(ArtifactCachePrefabAsset::from(asset)),
            ImportedAsset::Scene(asset) => Self::Scene(ArtifactCacheSceneAsset::from(asset)),
            ImportedAsset::Model(asset) => Self::Model(asset.clone()),
            ImportedAsset::Mesh(asset) => Self::Mesh(ArtifactCacheMeshAsset::from(asset)),
            ImportedAsset::AnimationSkeleton(asset) => Self::AnimationSkeleton(asset.clone()),
            ImportedAsset::AnimationClip(asset) => Self::AnimationClip(asset.clone()),
            ImportedAsset::AnimationSequence(asset) => Self::AnimationSequence(asset.clone()),
            ImportedAsset::AnimationGraph(asset) => Self::AnimationGraph(asset.clone()),
            ImportedAsset::AnimationStateMachine(asset) => {
                Self::AnimationStateMachine(asset.clone())
            }
            ImportedAsset::UiLayout(asset) => Self::UiLayout(
                ArtifactCacheUiAssetDocument::from_document(&asset.document)?,
            ),
            ImportedAsset::UiWidget(asset) => Self::UiWidget(
                ArtifactCacheUiAssetDocument::from_document(&asset.document)?,
            ),
            ImportedAsset::UiStyle(asset) => Self::UiStyle(
                ArtifactCacheUiAssetDocument::from_document(&asset.document)?,
            ),
            ImportedAsset::UiTheme(asset) => Self::UiTheme(asset.clone()),
            ImportedAsset::UiIcon(asset) => Self::UiIcon(asset.clone()),
            ImportedAsset::UiV2View(asset) => Self::UiV2View(
                ArtifactCacheUiV2AssetDocument::from_document(&asset.document)?,
            ),
            ImportedAsset::UiV2Component(asset) => Self::UiV2Component(
                ArtifactCacheUiV2AssetDocument::from_document(&asset.document)?,
            ),
            ImportedAsset::UiV2Style(asset) => Self::UiV2Style(
                ArtifactCacheUiV2AssetDocument::from_document(&asset.document)?,
            ),
        })
    }

    pub(super) fn into_imported(self) -> Result<ImportedAsset, AssetImportError> {
        Ok(match self {
            Self::Data(asset) => ImportedAsset::Data(asset.into_asset()?),
            Self::Texture(asset) => ImportedAsset::Texture(asset.into_asset()),
            Self::Shader(asset) => ImportedAsset::Shader(asset.into_asset()?),
            Self::Material(asset) => ImportedAsset::Material(asset.into_asset()?),
            Self::MaterialGraph(asset) => ImportedAsset::MaterialGraph(asset),
            Self::Sound(asset) => ImportedAsset::Sound(asset),
            Self::Font(asset) => ImportedAsset::Font(asset.into_asset()),
            Self::PhysicsMaterial(asset) => ImportedAsset::PhysicsMaterial(asset.into()),
            Self::NavMesh(asset) => ImportedAsset::NavMesh(asset),
            Self::NavigationSettings(asset) => ImportedAsset::NavigationSettings(asset),
            Self::Terrain(asset) => ImportedAsset::Terrain(asset),
            Self::TerrainLayerStack(asset) => ImportedAsset::TerrainLayerStack(asset),
            Self::TileSet(asset) => ImportedAsset::TileSet(asset),
            Self::TileMap(asset) => ImportedAsset::TileMap(asset),
            Self::Prefab(asset) => ImportedAsset::Prefab(asset.into_asset()?),
            Self::Scene(asset) => ImportedAsset::Scene(asset.into_asset()?),
            Self::Model(asset) => ImportedAsset::Model(asset),
            Self::Mesh(asset) => ImportedAsset::Mesh(asset.into_asset()),
            Self::AnimationSkeleton(asset) => ImportedAsset::AnimationSkeleton(asset),
            Self::AnimationClip(asset) => ImportedAsset::AnimationClip(asset),
            Self::AnimationSequence(asset) => ImportedAsset::AnimationSequence(asset),
            Self::AnimationGraph(asset) => ImportedAsset::AnimationGraph(asset),
            Self::AnimationStateMachine(asset) => ImportedAsset::AnimationStateMachine(asset),
            Self::UiLayout(asset) => ImportedAsset::UiLayout(asset.into_layout_asset()?),
            Self::UiWidget(asset) => ImportedAsset::UiWidget(asset.into_widget_asset()?),
            Self::UiStyle(asset) => ImportedAsset::UiStyle(asset.into_style_asset()?),
            Self::UiTheme(asset) => ImportedAsset::UiTheme(asset),
            Self::UiIcon(asset) => ImportedAsset::UiIcon(asset),
            Self::UiV2View(asset) => ImportedAsset::UiV2View(asset.into_view_asset()?),
            Self::UiV2Component(asset) => {
                ImportedAsset::UiV2Component(asset.into_component_asset()?)
            }
            Self::UiV2Style(asset) => ImportedAsset::UiV2Style(asset.into_style_asset()?),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheDataAsset {
    uri: AssetUri,
    format: crate::asset::DataAssetFormat,
    text: String,
    canonical_json: ArtifactCacheJsonValue,
}

impl From<&DataAsset> for ArtifactCacheDataAsset {
    fn from(asset: &DataAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            format: asset.format,
            text: asset.text.clone(),
            canonical_json: ArtifactCacheJsonValue::from_json(&asset.canonical_json),
        }
    }
}

impl ArtifactCacheDataAsset {
    fn into_asset(self) -> Result<DataAsset, AssetImportError> {
        Ok(DataAsset {
            uri: self.uri,
            format: self.format,
            text: self.text,
            canonical_json: self.canonical_json.into_json()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheTextureAsset {
    uri: AssetUri,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    payload: ArtifactCacheTexturePayload,
    descriptor: Option<crate::asset::TextureAssetDescriptor>,
}

impl From<&TextureAsset> for ArtifactCacheTextureAsset {
    fn from(asset: &TextureAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            width: asset.width,
            height: asset.height,
            rgba: asset.rgba.clone(),
            payload: ArtifactCacheTexturePayload::from(&asset.payload),
            descriptor: asset.descriptor.clone(),
        }
    }
}

impl ArtifactCacheTextureAsset {
    fn into_asset(self) -> TextureAsset {
        TextureAsset {
            uri: self.uri,
            width: self.width,
            height: self.height,
            rgba: self.rgba,
            payload: self.payload.into(),
            descriptor: self.descriptor,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ArtifactCacheTexturePayload {
    Rgba8,
    Container {
        format: String,
        bytes: Vec<u8>,
        mip_count: u32,
        array_layers: u32,
    },
}

impl From<&TexturePayload> for ArtifactCacheTexturePayload {
    fn from(payload: &TexturePayload) -> Self {
        match payload {
            TexturePayload::Rgba8 => Self::Rgba8,
            TexturePayload::Container {
                format,
                bytes,
                mip_count,
                array_layers,
            } => Self::Container {
                format: format.clone(),
                bytes: bytes.clone(),
                mip_count: *mip_count,
                array_layers: *array_layers,
            },
        }
    }
}

impl From<ArtifactCacheTexturePayload> for TexturePayload {
    fn from(payload: ArtifactCacheTexturePayload) -> Self {
        match payload {
            ArtifactCacheTexturePayload::Rgba8 => Self::Rgba8,
            ArtifactCacheTexturePayload::Container {
                format,
                bytes,
                mip_count,
                array_layers,
            } => Self::Container {
                format,
                bytes,
                mip_count,
                array_layers,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCachePrefabAsset {
    uri: AssetUri,
    name: String,
    scene: ArtifactCacheSceneAsset,
    exposed_properties: Vec<String>,
}

impl From<&PrefabAsset> for ArtifactCachePrefabAsset {
    fn from(asset: &PrefabAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            name: asset.name.clone(),
            scene: ArtifactCacheSceneAsset::from(&asset.scene),
            exposed_properties: asset.exposed_properties.clone(),
        }
    }
}

impl ArtifactCachePrefabAsset {
    fn into_asset(self) -> Result<PrefabAsset, AssetImportError> {
        Ok(PrefabAsset {
            uri: self.uri,
            name: self.name,
            scene: self.scene.into_asset()?,
            exposed_properties: self.exposed_properties,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCachePhysicsMaterialAsset {
    name: Option<String>,
    metadata: PhysicsMaterialMetadata,
}

impl From<&PhysicsMaterialAsset> for ArtifactCachePhysicsMaterialAsset {
    fn from(asset: &PhysicsMaterialAsset) -> Self {
        Self {
            name: asset.name.clone(),
            metadata: asset.metadata.clone(),
        }
    }
}

impl From<ArtifactCachePhysicsMaterialAsset> for PhysicsMaterialAsset {
    fn from(asset: ArtifactCachePhysicsMaterialAsset) -> Self {
        Self {
            name: asset.name,
            metadata: asset.metadata,
        }
    }
}
