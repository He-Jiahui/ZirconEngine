use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialTextureTransform;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialTextureSlotValue {
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub reference: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<RenderMaterialTextureTransform>,
    #[serde(default, skip_serializing_if = "is_default_uv_channel")]
    pub uv_channel: u32,
}

impl MaterialTextureSlotValue {
    pub fn new(reference: AssetReference) -> Self {
        Self {
            reference: Some(reference),
            fallback: None,
            transform: None,
            uv_channel: 0,
        }
    }

    pub fn texture_transform(&self) -> RenderMaterialTextureTransform {
        self.transform.unwrap_or_default()
    }

    pub fn texture_uv_channel(&self) -> u32 {
        self.uv_channel
    }
}

fn is_default_uv_channel(value: &u32) -> bool {
    *value == 0
}

pub(crate) fn is_standard_texture_slot_alias(slot: &str) -> bool {
    matches!(
        slot,
        "base_color"
            | "base_color_texture"
            | "albedo"
            | "diffuse"
            | "normal"
            | "normal_texture"
            | "metallic_roughness"
            | "metallic_roughness_texture"
            | "occlusion"
            | "occlusion_texture"
            | "emissive"
            | "emissive_texture"
            | "clearcoat_normal"
            | "clearcoat_normal_texture"
    )
}
