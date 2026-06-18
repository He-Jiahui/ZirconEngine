use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AlphaMode, AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset,
    AnimationSkeletonAsset, AnimationStateMachineAsset, AssetReference, AssetUri, DataAsset,
    FontAsset, ImportedAsset, MaterialAsset, MaterialGraphAsset, MaterialTextureSlotValue,
    ModelAsset, NavMeshAsset, NavigationSettingsAsset, PhysicsMaterialAsset, PrefabAsset,
    ShaderAsset, ShaderDependencyAsset, ShaderEntryPointAsset, ShaderImportRedirectAsset,
    ShaderMaterialPropertyAsset, ShaderSourceFileAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, SoundAsset, TerrainAsset, TerrainLayerStackAsset, TextureAsset,
    TexturePayload, TileMapAsset, TileSetAsset, UiIconAsset, UiLayoutAsset, UiStyleAsset,
    UiThemeAsset, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset, UiWidgetAsset,
};
use crate::core::framework::physics::PhysicsMaterialMetadata;
use crate::core::framework::render::{
    RenderMaterialTextureTransform, RenderShaderDefinitionValue,
    RenderShaderPipelineLayoutDescriptor,
};

mod json_value;
mod mesh;
mod scene;
mod toml_value;

use json_value::ArtifactCacheJsonValue;
use mesh::ArtifactCacheMeshAsset;
use scene::ArtifactCacheSceneAsset;
use toml_value::{
    cache_table_like_to_toml, cache_table_to_toml, toml_table_like_to_cache, toml_table_to_cache,
    ArtifactCacheTomlTable, ArtifactCacheTomlValue,
};

/// Bincode cache wire type. It keeps authoring-friendly serde shapes such as
/// TOML values and flattened fields out of runtime library artifacts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum ArtifactCacheAsset {
    Data(ArtifactCacheDataAsset),
    Texture(ArtifactCacheTextureAsset),
    Shader(ArtifactCacheShaderAsset),
    Material(ArtifactCacheMaterialAsset),
    MaterialGraph(MaterialGraphAsset),
    Sound(SoundAsset),
    Font(FontAsset),
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
    UiLayout(UiLayoutAsset),
    UiWidget(UiWidgetAsset),
    UiStyle(UiStyleAsset),
    UiTheme(UiThemeAsset),
    UiIcon(UiIconAsset),
    UiV2View(UiV2ViewAsset),
    UiV2Component(UiV2ComponentAsset),
    UiV2Style(UiV2StyleAsset),
}

impl ArtifactCacheAsset {
    pub(super) fn from_imported(asset: &ImportedAsset) -> Self {
        match asset {
            ImportedAsset::Data(asset) => Self::Data(ArtifactCacheDataAsset::from(asset)),
            ImportedAsset::Texture(asset) => Self::Texture(ArtifactCacheTextureAsset::from(asset)),
            ImportedAsset::Shader(asset) => Self::Shader(ArtifactCacheShaderAsset::from(asset)),
            ImportedAsset::Material(asset) => {
                Self::Material(ArtifactCacheMaterialAsset::from(asset))
            }
            ImportedAsset::MaterialGraph(asset) => Self::MaterialGraph(asset.clone()),
            ImportedAsset::Sound(asset) => Self::Sound(asset.clone()),
            ImportedAsset::Font(asset) => Self::Font(asset.clone()),
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
            ImportedAsset::UiLayout(asset) => Self::UiLayout(asset.clone()),
            ImportedAsset::UiWidget(asset) => Self::UiWidget(asset.clone()),
            ImportedAsset::UiStyle(asset) => Self::UiStyle(asset.clone()),
            ImportedAsset::UiTheme(asset) => Self::UiTheme(asset.clone()),
            ImportedAsset::UiIcon(asset) => Self::UiIcon(asset.clone()),
            ImportedAsset::UiV2View(asset) => Self::UiV2View(asset.clone()),
            ImportedAsset::UiV2Component(asset) => Self::UiV2Component(asset.clone()),
            ImportedAsset::UiV2Style(asset) => Self::UiV2Style(asset.clone()),
        }
    }

    pub(super) fn into_imported(self) -> Result<ImportedAsset, String> {
        Ok(match self {
            Self::Data(asset) => ImportedAsset::Data(asset.into()),
            Self::Texture(asset) => ImportedAsset::Texture(asset.into_asset()),
            Self::Shader(asset) => ImportedAsset::Shader(asset.into_asset()?),
            Self::Material(asset) => ImportedAsset::Material(asset.into_asset()?),
            Self::MaterialGraph(asset) => ImportedAsset::MaterialGraph(asset),
            Self::Sound(asset) => ImportedAsset::Sound(asset),
            Self::Font(asset) => ImportedAsset::Font(asset),
            Self::PhysicsMaterial(asset) => ImportedAsset::PhysicsMaterial(asset.into()),
            Self::NavMesh(asset) => ImportedAsset::NavMesh(asset),
            Self::NavigationSettings(asset) => ImportedAsset::NavigationSettings(asset),
            Self::Terrain(asset) => ImportedAsset::Terrain(asset),
            Self::TerrainLayerStack(asset) => ImportedAsset::TerrainLayerStack(asset),
            Self::TileSet(asset) => ImportedAsset::TileSet(asset),
            Self::TileMap(asset) => ImportedAsset::TileMap(asset),
            Self::Prefab(asset) => ImportedAsset::Prefab(asset.into_asset()?),
            Self::Scene(asset) => ImportedAsset::Scene(asset.into_asset()),
            Self::Model(asset) => ImportedAsset::Model(asset),
            Self::Mesh(asset) => ImportedAsset::Mesh(asset.into_asset()),
            Self::AnimationSkeleton(asset) => ImportedAsset::AnimationSkeleton(asset),
            Self::AnimationClip(asset) => ImportedAsset::AnimationClip(asset),
            Self::AnimationSequence(asset) => ImportedAsset::AnimationSequence(asset),
            Self::AnimationGraph(asset) => ImportedAsset::AnimationGraph(asset),
            Self::AnimationStateMachine(asset) => ImportedAsset::AnimationStateMachine(asset),
            Self::UiLayout(asset) => ImportedAsset::UiLayout(asset),
            Self::UiWidget(asset) => ImportedAsset::UiWidget(asset),
            Self::UiStyle(asset) => ImportedAsset::UiStyle(asset),
            Self::UiTheme(asset) => ImportedAsset::UiTheme(asset),
            Self::UiIcon(asset) => ImportedAsset::UiIcon(asset),
            Self::UiV2View(asset) => ImportedAsset::UiV2View(asset),
            Self::UiV2Component(asset) => ImportedAsset::UiV2Component(asset),
            Self::UiV2Style(asset) => ImportedAsset::UiV2Style(asset),
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

impl From<ArtifactCacheDataAsset> for DataAsset {
    fn from(asset: ArtifactCacheDataAsset) -> Self {
        Self {
            uri: asset.uri,
            format: asset.format,
            text: asset.text,
            canonical_json: asset.canonical_json.into_json(),
        }
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
    fn into_asset(self) -> Result<PrefabAsset, String> {
        Ok(PrefabAsset {
            uri: self.uri,
            name: self.name,
            scene: self.scene.into_asset(),
            exposed_properties: self.exposed_properties,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheMaterialAsset {
    name: Option<String>,
    shader: AssetReference,
    base_color: [f32; 4],
    base_color_texture: Option<AssetReference>,
    normal_texture: Option<AssetReference>,
    metallic: f32,
    roughness: f32,
    metallic_roughness_texture: Option<AssetReference>,
    occlusion_texture: Option<AssetReference>,
    emissive: [f32; 3],
    emissive_texture: Option<AssetReference>,
    alpha_mode: ArtifactCacheAlphaMode,
    double_sided: bool,
    property_values: BTreeMap<String, ArtifactCacheTomlValue>,
    texture_slots: BTreeMap<String, ArtifactCacheMaterialTextureSlotValue>,
    validation_diagnostics: Vec<String>,
}

impl From<&MaterialAsset> for ArtifactCacheMaterialAsset {
    fn from(asset: &MaterialAsset) -> Self {
        Self {
            name: asset.name.clone(),
            shader: asset.shader.clone(),
            base_color: asset.base_color,
            base_color_texture: asset.base_color_texture.clone(),
            normal_texture: asset.normal_texture.clone(),
            metallic: asset.metallic,
            roughness: asset.roughness,
            metallic_roughness_texture: asset.metallic_roughness_texture.clone(),
            occlusion_texture: asset.occlusion_texture.clone(),
            emissive: asset.emissive,
            emissive_texture: asset.emissive_texture.clone(),
            alpha_mode: ArtifactCacheAlphaMode::from(&asset.alpha_mode),
            double_sided: asset.double_sided,
            property_values: toml_table_like_to_cache(&asset.property_values),
            texture_slots: asset
                .texture_slots
                .iter()
                .map(|(slot, value)| {
                    (
                        slot.clone(),
                        ArtifactCacheMaterialTextureSlotValue::from(value),
                    )
                })
                .collect(),
            validation_diagnostics: asset.validation_diagnostics.clone(),
        }
    }
}

impl ArtifactCacheMaterialAsset {
    fn into_asset(self) -> Result<MaterialAsset, String> {
        Ok(MaterialAsset {
            name: self.name,
            shader: self.shader,
            base_color: self.base_color,
            base_color_texture: self.base_color_texture,
            normal_texture: self.normal_texture,
            metallic: self.metallic,
            roughness: self.roughness,
            metallic_roughness_texture: self.metallic_roughness_texture,
            occlusion_texture: self.occlusion_texture,
            emissive: self.emissive,
            emissive_texture: self.emissive_texture,
            alpha_mode: self.alpha_mode.into(),
            double_sided: self.double_sided,
            property_values: cache_table_like_to_toml(self.property_values)?,
            texture_slots: self
                .texture_slots
                .into_iter()
                .map(|(slot, value)| (slot, value.into()))
                .collect(),
            validation_diagnostics: self.validation_diagnostics,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheShaderAsset {
    uri: AssetUri,
    source_language: ShaderSourceLanguage,
    source: String,
    wgsl_source: String,
    import_path: Option<String>,
    entry_points: Vec<ShaderEntryPointAsset>,
    dependencies: Vec<ShaderDependencyAsset>,
    source_files: Vec<ShaderSourceFileAsset>,
    imports: Vec<ArtifactCacheShaderImportRedirectAsset>,
    shader_defs: Vec<ArtifactCacheRenderShaderDefinitionValue>,
    property_schema: Vec<ArtifactCacheShaderMaterialPropertyAsset>,
    texture_slots: Vec<ArtifactCacheShaderTextureSlotAsset>,
    editor: ArtifactCacheTomlTable,
    pipeline_layout: RenderShaderPipelineLayoutDescriptor,
    validation_diagnostics: Vec<String>,
}

impl From<&ShaderAsset> for ArtifactCacheShaderAsset {
    fn from(asset: &ShaderAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            source_language: asset.source_language,
            source: asset.source.clone(),
            wgsl_source: asset.wgsl_source.clone(),
            import_path: asset.import_path.clone(),
            entry_points: asset.entry_points.clone(),
            dependencies: asset.dependencies.clone(),
            source_files: asset.source_files.clone(),
            imports: asset
                .imports
                .iter()
                .map(ArtifactCacheShaderImportRedirectAsset::from)
                .collect(),
            shader_defs: asset
                .shader_defs
                .iter()
                .map(ArtifactCacheRenderShaderDefinitionValue::from)
                .collect(),
            property_schema: asset
                .property_schema
                .iter()
                .map(ArtifactCacheShaderMaterialPropertyAsset::from)
                .collect(),
            texture_slots: asset
                .texture_slots
                .iter()
                .map(ArtifactCacheShaderTextureSlotAsset::from)
                .collect(),
            editor: toml_table_to_cache(&asset.editor),
            pipeline_layout: asset.pipeline_layout.clone(),
            validation_diagnostics: asset.validation_diagnostics.clone(),
        }
    }
}

impl ArtifactCacheShaderAsset {
    fn into_asset(self) -> Result<ShaderAsset, String> {
        Ok(ShaderAsset {
            uri: self.uri,
            source_language: self.source_language,
            source: self.source,
            wgsl_source: self.wgsl_source,
            import_path: self.import_path,
            entry_points: self.entry_points,
            dependencies: self.dependencies,
            source_files: self.source_files,
            imports: self.imports.into_iter().map(Into::into).collect(),
            shader_defs: self
                .shader_defs
                .into_iter()
                .map(RenderShaderDefinitionValue::from)
                .collect(),
            property_schema: self
                .property_schema
                .into_iter()
                .map(ArtifactCacheShaderMaterialPropertyAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
            texture_slots: self.texture_slots.into_iter().map(Into::into).collect(),
            editor: cache_table_to_toml(self.editor)?,
            pipeline_layout: self.pipeline_layout,
            validation_diagnostics: self.validation_diagnostics,
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheAlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

impl From<&AlphaMode> for ArtifactCacheAlphaMode {
    fn from(value: &AlphaMode) -> Self {
        match value {
            AlphaMode::Opaque => Self::Opaque,
            AlphaMode::Mask { cutoff } => Self::Mask { cutoff: *cutoff },
            AlphaMode::Blend => Self::Blend,
        }
    }
}

impl From<ArtifactCacheAlphaMode> for AlphaMode {
    fn from(value: ArtifactCacheAlphaMode) -> Self {
        match value {
            ArtifactCacheAlphaMode::Opaque => Self::Opaque,
            ArtifactCacheAlphaMode::Mask { cutoff } => Self::Mask { cutoff },
            ArtifactCacheAlphaMode::Blend => Self::Blend,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheMaterialTextureSlotValue {
    reference: Option<AssetReference>,
    fallback: Option<String>,
    transform: Option<RenderMaterialTextureTransform>,
    uv_channel: u32,
}

impl From<&MaterialTextureSlotValue> for ArtifactCacheMaterialTextureSlotValue {
    fn from(value: &MaterialTextureSlotValue) -> Self {
        Self {
            reference: value.reference.clone(),
            fallback: value.fallback.clone(),
            transform: value.transform,
            uv_channel: value.uv_channel,
        }
    }
}

impl From<ArtifactCacheMaterialTextureSlotValue> for MaterialTextureSlotValue {
    fn from(value: ArtifactCacheMaterialTextureSlotValue) -> Self {
        Self {
            reference: value.reference,
            fallback: value.fallback,
            transform: value.transform,
            uv_channel: value.uv_channel,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheShaderImportRedirectAsset {
    source: String,
    redirect: Option<AssetReference>,
}

impl From<&ShaderImportRedirectAsset> for ArtifactCacheShaderImportRedirectAsset {
    fn from(value: &ShaderImportRedirectAsset) -> Self {
        Self {
            source: value.source.clone(),
            redirect: value.redirect.clone(),
        }
    }
}

impl From<ArtifactCacheShaderImportRedirectAsset> for ShaderImportRedirectAsset {
    fn from(value: ArtifactCacheShaderImportRedirectAsset) -> Self {
        Self {
            source: value.source,
            redirect: value.redirect,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheShaderMaterialPropertyAsset {
    name: String,
    kind: String,
    required: bool,
    default: Option<ArtifactCacheTomlValue>,
    editor: BTreeMap<String, String>,
}

impl From<&ShaderMaterialPropertyAsset> for ArtifactCacheShaderMaterialPropertyAsset {
    fn from(value: &ShaderMaterialPropertyAsset) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind.clone(),
            required: value.required,
            default: value
                .default
                .as_ref()
                .map(ArtifactCacheTomlValue::from_toml),
            editor: value.editor.clone(),
        }
    }
}

impl ArtifactCacheShaderMaterialPropertyAsset {
    fn into_asset(self) -> Result<ShaderMaterialPropertyAsset, String> {
        Ok(ShaderMaterialPropertyAsset {
            name: self.name,
            kind: self.kind,
            required: self.required,
            default: self
                .default
                .map(ArtifactCacheTomlValue::into_toml)
                .transpose()?,
            editor: self.editor,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheShaderTextureSlotAsset {
    name: String,
    kind: String,
    required: bool,
    default: Option<String>,
    sampler: Option<String>,
    group: Option<String>,
    label: Option<String>,
    editor: BTreeMap<String, String>,
}

impl From<&ShaderTextureSlotAsset> for ArtifactCacheShaderTextureSlotAsset {
    fn from(value: &ShaderTextureSlotAsset) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind.clone(),
            required: value.required,
            default: value.default.clone(),
            sampler: value.sampler.clone(),
            group: value.group.clone(),
            label: value.label.clone(),
            editor: value.editor.clone(),
        }
    }
}

impl From<ArtifactCacheShaderTextureSlotAsset> for ShaderTextureSlotAsset {
    fn from(value: ArtifactCacheShaderTextureSlotAsset) -> Self {
        Self {
            name: value.name,
            kind: value.kind,
            required: value.required,
            default: value.default,
            sampler: value.sampler,
            group: value.group,
            label: value.label,
            editor: value.editor,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ArtifactCacheRenderShaderDefinitionValue {
    Bool { name: String, value: bool },
    Int { name: String, value: i32 },
    UInt { name: String, value: u32 },
}

impl From<&RenderShaderDefinitionValue> for ArtifactCacheRenderShaderDefinitionValue {
    fn from(value: &RenderShaderDefinitionValue) -> Self {
        match value {
            RenderShaderDefinitionValue::Bool { name, value } => Self::Bool {
                name: name.clone(),
                value: *value,
            },
            RenderShaderDefinitionValue::Int { name, value } => Self::Int {
                name: name.clone(),
                value: *value,
            },
            RenderShaderDefinitionValue::UInt { name, value } => Self::UInt {
                name: name.clone(),
                value: *value,
            },
        }
    }
}

impl From<ArtifactCacheRenderShaderDefinitionValue> for RenderShaderDefinitionValue {
    fn from(value: ArtifactCacheRenderShaderDefinitionValue) -> Self {
        match value {
            ArtifactCacheRenderShaderDefinitionValue::Bool { name, value } => {
                Self::bool(name, value)
            }
            ArtifactCacheRenderShaderDefinitionValue::Int { name, value } => Self::int(name, value),
            ArtifactCacheRenderShaderDefinitionValue::UInt { name, value } => {
                Self::uint(name, value)
            }
        }
    }
}
