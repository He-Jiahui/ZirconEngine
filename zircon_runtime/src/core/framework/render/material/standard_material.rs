use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderQueueValue;
use crate::core::framework::render::advanced_lighting::StandardPbrMaterialFeatures;
use crate::core::resource::AssetReference;

use super::{
    RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialFallbackPolicy,
    RenderMaterialLightingModel, RenderMaterialTextureTransform,
};

pub const STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandardMaterialDescriptor {
    pub name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub base_color: [f32; 4],
    pub base_color_texture: Option<AssetReference>,
    #[serde(default)]
    pub base_color_texture_transform: RenderMaterialTextureTransform,
    #[serde(default)]
    pub base_color_texture_uv_channel: u32,
    pub normal_texture: Option<AssetReference>,
    #[serde(default)]
    pub normal_texture_transform: RenderMaterialTextureTransform,
    #[serde(default)]
    pub normal_texture_uv_channel: u32,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<AssetReference>,
    #[serde(default)]
    pub metallic_roughness_texture_transform: RenderMaterialTextureTransform,
    #[serde(default)]
    pub metallic_roughness_texture_uv_channel: u32,
    pub occlusion_texture: Option<AssetReference>,
    #[serde(default)]
    pub occlusion_texture_transform: RenderMaterialTextureTransform,
    #[serde(default)]
    pub occlusion_texture_uv_channel: u32,
    pub emissive: [f32; 3],
    pub emissive_texture: Option<AssetReference>,
    #[serde(default)]
    pub emissive_texture_transform: RenderMaterialTextureTransform,
    #[serde(default)]
    pub emissive_texture_uv_channel: u32,
    pub alpha_mode: RenderMaterialAlphaMode,
    #[serde(default)]
    pub lighting_model: RenderMaterialLightingModel,
    pub unlit: bool,
    pub double_sided: bool,
    #[serde(default = "default_cast_shadows")]
    pub cast_shadows: bool,
    #[serde(default = "default_receive_shadows")]
    pub receive_shadows: bool,
    #[serde(default)]
    pub render_queue: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_queue_value: Option<RenderQueueValue>,
    #[serde(default)]
    pub material_queue: i32,
    #[serde(default)]
    pub depth_bias: f32,
    #[serde(default)]
    pub taa_reactive_mask_strength: f32,
    #[serde(
        default,
        skip_serializing_if = "StandardPbrMaterialFeatures::is_default"
    )]
    pub advanced_features: StandardPbrMaterialFeatures,
    #[serde(default)]
    pub subsurface_profile_index: u32,
    pub fallback_policy: RenderMaterialFallbackPolicy,
}

impl StandardMaterialDescriptor {
    pub fn resolved_render_queue_value(&self) -> RenderQueueValue {
        self.render_queue_value.unwrap_or_else(|| {
            RenderQueueValue::from_authored_queue(&self.alpha_mode, self.render_queue)
        })
    }
}

fn default_cast_shadows() -> bool {
    true
}

fn default_receive_shadows() -> bool {
    true
}
