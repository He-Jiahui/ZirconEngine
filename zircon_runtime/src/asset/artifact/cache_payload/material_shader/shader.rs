use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetImportError, AssetReference, AssetUri, ShaderAsset, ShaderDependencyAsset,
    ShaderEntryPointAsset, ShaderImportRedirectAsset, ShaderMaterialPropertyAsset,
    ShaderOptionAsset, ShaderSourceFileAsset, ShaderSourceLanguage, ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    MaterialOptionTable, MaterialPropertyKind, MaterialPropertyLayout, MaterialPropertySlotRef,
    MaterialTextureBindingRef, RenderShaderDefinitionValue, RenderShaderPipelineLayoutDescriptor,
    ShaderAssetKind, ShaderBlendMode, ShaderCullMode, ShaderDepthCompare, ShaderQueueDescriptor,
    ShaderRenderStateDescriptor, ShaderResourceAccess, ShaderResourceDescriptor,
    ShaderResourceKind,
};

use super::super::toml_value::{
    ArtifactCacheTomlTable, ArtifactCacheTomlValue, cache_table_to_toml, toml_table_to_cache,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in super::super) struct ArtifactCacheShaderAsset {
    uri: AssetUri,
    #[serde(default = "default_artifact_shader_asset_kind")]
    kind: ShaderAssetKind,
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
    #[serde(default)]
    options: Vec<ArtifactCacheShaderOptionAsset>,
    texture_slots: Vec<ArtifactCacheShaderTextureSlotAsset>,
    #[serde(default)]
    shading_model: Option<String>,
    #[serde(default)]
    render_state: ArtifactCacheShaderRenderStateDescriptor,
    #[serde(default)]
    queue: Option<ShaderQueueDescriptor>,
    #[serde(default)]
    disabled_passes: Vec<String>,
    #[serde(default)]
    resources: Vec<ArtifactCacheShaderResourceDescriptor>,
    #[serde(default)]
    material_property_layout: ArtifactCacheMaterialPropertyLayout,
    #[serde(default)]
    material_option_table: MaterialOptionTable,
    #[serde(default)]
    generated_material_wgsl: String,
    editor: ArtifactCacheTomlTable,
    pipeline_layout: RenderShaderPipelineLayoutDescriptor,
    validation_diagnostics: Vec<String>,
}

impl From<&ShaderAsset> for ArtifactCacheShaderAsset {
    fn from(asset: &ShaderAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            kind: asset.kind,
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
            options: asset
                .options
                .iter()
                .map(ArtifactCacheShaderOptionAsset::from)
                .collect(),
            texture_slots: asset
                .texture_slots
                .iter()
                .map(ArtifactCacheShaderTextureSlotAsset::from)
                .collect(),
            shading_model: asset.shading_model.clone(),
            render_state: ArtifactCacheShaderRenderStateDescriptor::from(&asset.render_state),
            queue: asset.queue,
            disabled_passes: asset.disabled_passes.clone(),
            resources: asset
                .resources
                .iter()
                .map(ArtifactCacheShaderResourceDescriptor::from)
                .collect(),
            material_property_layout: ArtifactCacheMaterialPropertyLayout::from(
                &asset.material_property_layout,
            ),
            material_option_table: asset.material_option_table.clone(),
            generated_material_wgsl: asset.generated_material_wgsl.clone(),
            editor: toml_table_to_cache(&asset.editor),
            pipeline_layout: asset.pipeline_layout.clone(),
            validation_diagnostics: asset.validation_diagnostics.clone(),
        }
    }
}

impl ArtifactCacheShaderAsset {
    pub(in super::super) fn into_asset(self) -> Result<ShaderAsset, AssetImportError> {
        let mut asset = ShaderAsset {
            uri: self.uri,
            kind: self.kind,
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
            options: self
                .options
                .into_iter()
                .map(ArtifactCacheShaderOptionAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
            texture_slots: self.texture_slots.into_iter().map(Into::into).collect(),
            shading_model: self.shading_model,
            render_state: self.render_state.into(),
            queue: self.queue,
            disabled_passes: self.disabled_passes,
            resources: self.resources.into_iter().map(Into::into).collect(),
            material_property_layout: self.material_property_layout.into(),
            material_option_table: self.material_option_table,
            generated_material_wgsl: self.generated_material_wgsl,
            editor: cache_table_to_toml(self.editor)?,
            pipeline_layout: self.pipeline_layout,
            validation_diagnostics: self.validation_diagnostics,
        };
        asset.regenerate_material_artifact();
        Ok(asset)
    }
}

fn default_artifact_shader_asset_kind() -> ShaderAssetKind {
    ShaderAssetKind::Surface
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
struct ArtifactCacheShaderOptionAsset {
    name: String,
    kind: String,
    default: Option<ArtifactCacheTomlValue>,
    editor: BTreeMap<String, String>,
}

impl From<&ShaderOptionAsset> for ArtifactCacheShaderOptionAsset {
    fn from(value: &ShaderOptionAsset) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind.clone(),
            default: value
                .default
                .as_ref()
                .map(ArtifactCacheTomlValue::from_toml),
            editor: value.editor.clone(),
        }
    }
}

impl ArtifactCacheShaderOptionAsset {
    fn into_asset(self) -> Result<ShaderOptionAsset, AssetImportError> {
        Ok(ShaderOptionAsset {
            name: self.name,
            kind: self.kind,
            default: self
                .default
                .map(ArtifactCacheTomlValue::into_toml)
                .transpose()?,
            editor: self.editor,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheShaderMaterialPropertyAsset {
    name: String,
    kind: MaterialPropertyKind,
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
    fn into_asset(self) -> Result<ShaderMaterialPropertyAsset, AssetImportError> {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheShaderRenderStateDescriptor {
    cull_mode: Option<ShaderCullMode>,
    depth_compare: Option<ShaderDepthCompare>,
    depth_write: Option<bool>,
    blend: Option<ShaderBlendMode>,
}

impl From<&ShaderRenderStateDescriptor> for ArtifactCacheShaderRenderStateDescriptor {
    fn from(value: &ShaderRenderStateDescriptor) -> Self {
        Self {
            cull_mode: value.cull_mode,
            depth_compare: value.depth_compare,
            depth_write: value.depth_write,
            blend: value.blend,
        }
    }
}

impl From<ArtifactCacheShaderRenderStateDescriptor> for ShaderRenderStateDescriptor {
    fn from(value: ArtifactCacheShaderRenderStateDescriptor) -> Self {
        Self {
            cull_mode: value.cull_mode,
            depth_compare: value.depth_compare,
            depth_write: value.depth_write,
            blend: value.blend,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheShaderResourceDescriptor {
    name: String,
    kind: ShaderResourceKind,
    access: Option<ShaderResourceAccess>,
}

impl From<&ShaderResourceDescriptor> for ArtifactCacheShaderResourceDescriptor {
    fn from(value: &ShaderResourceDescriptor) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind,
            access: value.access,
        }
    }
}

impl From<ArtifactCacheShaderResourceDescriptor> for ShaderResourceDescriptor {
    fn from(value: ArtifactCacheShaderResourceDescriptor) -> Self {
        Self {
            name: value.name,
            kind: value.kind,
            access: value.access,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheMaterialPropertyLayout {
    properties: Vec<MaterialPropertySlotRef>,
    f32_slot_count: u16,
    u32_slot_count: u16,
    packed_size: u32,
    texture_bindings: Vec<ArtifactCacheMaterialTextureBindingRef>,
    layout_hash: u64,
}

impl From<&MaterialPropertyLayout> for ArtifactCacheMaterialPropertyLayout {
    fn from(value: &MaterialPropertyLayout) -> Self {
        Self {
            properties: value.properties.clone(),
            f32_slot_count: value.f32_slot_count,
            u32_slot_count: value.u32_slot_count,
            packed_size: value.packed_size,
            texture_bindings: value
                .texture_bindings
                .iter()
                .map(ArtifactCacheMaterialTextureBindingRef::from)
                .collect(),
            layout_hash: value.layout_hash,
        }
    }
}

impl From<ArtifactCacheMaterialPropertyLayout> for MaterialPropertyLayout {
    fn from(value: ArtifactCacheMaterialPropertyLayout) -> Self {
        Self {
            properties: value.properties,
            f32_slot_count: value.f32_slot_count,
            u32_slot_count: value.u32_slot_count,
            packed_size: value.packed_size,
            texture_bindings: value.texture_bindings.into_iter().map(Into::into).collect(),
            layout_hash: value.layout_hash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactCacheMaterialTextureBindingRef {
    name: String,
    kind: String,
    texture_binding: u16,
    sampler_binding: u16,
    option: Option<String>,
    has_st_transform: bool,
}

impl From<&MaterialTextureBindingRef> for ArtifactCacheMaterialTextureBindingRef {
    fn from(value: &MaterialTextureBindingRef) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind.clone(),
            texture_binding: value.texture_binding,
            sampler_binding: value.sampler_binding,
            option: value.option.clone(),
            has_st_transform: value.has_st_transform,
        }
    }
}

impl From<ArtifactCacheMaterialTextureBindingRef> for MaterialTextureBindingRef {
    fn from(value: ArtifactCacheMaterialTextureBindingRef) -> Self {
        Self {
            name: value.name,
            kind: value.kind,
            texture_binding: value.texture_binding,
            sampler_binding: value.sampler_binding,
            option: value.option,
            has_st_transform: value.has_st_transform,
        }
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
    option: Option<String>,
    st: bool,
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
            option: value.option.clone(),
            st: value.st,
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
            option: value.option,
            st: value.st,
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
