use std::collections::BTreeMap;

#[cfg(test)]
use crate::core::framework::render::RenderMaterialLightingModel;
use crate::core::framework::render::{
    RenderMaterialPropertyUniformPayload, RenderMaterialPropertyValue,
    RenderMaterialReadinessReport, RenderMaterialTextureTransform, RenderQueueValue,
    ShadingModelId,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::PipelineKey;

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct MaterialCaptureSeed {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    #[cfg(test)]
    pub(crate) double_sided: bool,
    #[cfg(test)]
    pub(crate) alpha_blend: bool,
    #[cfg(test)]
    pub(crate) alpha_cutoff: Option<f32>,
    #[cfg(test)]
    pub(crate) lighting_model: RenderMaterialLightingModel,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    pub(crate) cast_shadows: bool,
    pub(crate) receive_shadows: bool,
    pub(crate) taa_reactive_mask_strength: f32,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) base_color_texture_transform: RenderMaterialTextureTransform,
    pub(crate) base_color_texture_uv_channel: u32,
    pub(crate) normal_texture: Option<ResourceId>,
    pub(crate) normal_texture_transform: RenderMaterialTextureTransform,
    pub(crate) normal_texture_uv_channel: u32,
    pub(crate) metallic_roughness_texture: Option<ResourceId>,
    pub(crate) metallic_roughness_texture_transform: RenderMaterialTextureTransform,
    pub(crate) metallic_roughness_texture_uv_channel: u32,
    pub(crate) occlusion_texture: Option<ResourceId>,
    pub(crate) occlusion_texture_transform: RenderMaterialTextureTransform,
    pub(crate) occlusion_texture_uv_channel: u32,
    pub(crate) emissive_texture: Option<ResourceId>,
    pub(crate) emissive_texture_transform: RenderMaterialTextureTransform,
    pub(crate) emissive_texture_uv_channel: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    #[cfg(test)]
    pub(crate) double_sided: bool,
    #[cfg(test)]
    pub(crate) alpha_blend: bool,
    #[cfg(test)]
    pub(crate) alpha_cutoff: Option<f32>,
    #[cfg(test)]
    pub(crate) lighting_model: RenderMaterialLightingModel,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    pub(crate) cast_shadows: bool,
    pub(crate) receive_shadows: bool,
    pub(crate) render_queue: i32,
    pub(crate) render_queue_value: Option<RenderQueueValue>,
    pub(crate) material_queue: i32,
    pub(crate) depth_bias: f32,
    pub(crate) taa_reactive_mask_strength: f32,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) base_color_texture_transform: RenderMaterialTextureTransform,
    pub(crate) base_color_texture_uv_channel: u32,
    pub(crate) normal_texture: Option<ResourceId>,
    pub(crate) normal_texture_transform: RenderMaterialTextureTransform,
    pub(crate) normal_texture_uv_channel: u32,
    pub(crate) metallic_roughness_texture: Option<ResourceId>,
    pub(crate) metallic_roughness_texture_transform: RenderMaterialTextureTransform,
    pub(crate) metallic_roughness_texture_uv_channel: u32,
    pub(crate) occlusion_texture: Option<ResourceId>,
    pub(crate) occlusion_texture_transform: RenderMaterialTextureTransform,
    pub(crate) occlusion_texture_uv_channel: u32,
    pub(crate) emissive_texture: Option<ResourceId>,
    pub(crate) emissive_texture_transform: RenderMaterialTextureTransform,
    pub(crate) emissive_texture_uv_channel: u32,
    pub(crate) shader_property_values: BTreeMap<String, RenderMaterialPropertyValue>,
    pub(crate) shader_property_uniform_payload: RenderMaterialPropertyUniformPayload,
    pub(crate) non_standard_texture_slots: BTreeMap<String, Option<ResourceId>>,
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) readiness_report: RenderMaterialReadinessReport,
}

#[cfg(test)]
impl MaterialRuntime {
    pub(crate) fn capture_seed(&self) -> MaterialCaptureSeed {
        MaterialCaptureSeed {
            base_color: self.base_color,
            emissive: self.emissive,
            metallic: self.metallic,
            roughness: self.roughness,
            double_sided: self.double_sided,
            alpha_blend: self.alpha_blend,
            alpha_cutoff: self.alpha_cutoff,
            lighting_model: self.lighting_model.clone(),
            shading_model_id: self.shading_model_id,
            unlit: self.unlit,
            cast_shadows: self.cast_shadows,
            receive_shadows: self.receive_shadows,
            taa_reactive_mask_strength: self.taa_reactive_mask_strength,
            base_color_texture: self.base_color_texture,
            base_color_texture_transform: self.base_color_texture_transform,
            base_color_texture_uv_channel: self.base_color_texture_uv_channel,
            normal_texture: self.normal_texture,
            normal_texture_transform: self.normal_texture_transform,
            normal_texture_uv_channel: self.normal_texture_uv_channel,
            metallic_roughness_texture: self.metallic_roughness_texture,
            metallic_roughness_texture_transform: self.metallic_roughness_texture_transform,
            metallic_roughness_texture_uv_channel: self.metallic_roughness_texture_uv_channel,
            occlusion_texture: self.occlusion_texture,
            occlusion_texture_transform: self.occlusion_texture_transform,
            occlusion_texture_uv_channel: self.occlusion_texture_uv_channel,
            emissive_texture: self.emissive_texture,
            emissive_texture_transform: self.emissive_texture_transform,
            emissive_texture_uv_channel: self.emissive_texture_uv_channel,
        }
    }
}
