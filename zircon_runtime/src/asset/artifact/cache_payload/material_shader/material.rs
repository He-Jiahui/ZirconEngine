use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AlphaMode, AssetImportError, AssetReference, MaterialAsset, MaterialTextureSlotValue,
    ZMaterialQueueOverride,
};
use crate::core::framework::render::RenderMaterialTextureTransform;

use super::super::toml_value::{
    ArtifactCacheTomlValue, cache_table_like_to_toml, toml_table_like_to_cache,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in super::super) struct ArtifactCacheMaterialAsset {
    name: Option<String>,
    shader: AssetReference,
    #[serde(default)]
    parent: Option<AssetReference>,
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
    #[serde(default)]
    options: BTreeMap<String, ArtifactCacheTomlValue>,
    #[serde(default)]
    queue: Option<ZMaterialQueueOverride>,
    validation_diagnostics: Vec<String>,
}

impl From<&MaterialAsset> for ArtifactCacheMaterialAsset {
    fn from(asset: &MaterialAsset) -> Self {
        Self {
            name: asset.name.clone(),
            shader: asset.shader.clone(),
            parent: asset.parent.clone(),
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
            options: toml_table_like_to_cache(&asset.options),
            queue: asset.queue,
            validation_diagnostics: asset.validation_diagnostics.clone(),
        }
    }
}

impl ArtifactCacheMaterialAsset {
    pub(in super::super) fn into_asset(self) -> Result<MaterialAsset, AssetImportError> {
        Ok(MaterialAsset {
            name: self.name,
            shader: self.shader,
            parent: self.parent,
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
            options: cache_table_like_to_toml(self.options)?,
            queue: self.queue,
            validation_diagnostics: self.validation_diagnostics,
        })
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
