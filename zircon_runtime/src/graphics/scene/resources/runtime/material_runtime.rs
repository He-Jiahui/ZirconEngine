use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderMaterialLightingModel, RenderMaterialPropertyUniformPayload, RenderMaterialPropertyValue,
    RenderMaterialReadinessReport, RenderMaterialTextureTransform, RenderQueueValue,
    ShadingModelId, StandardPbrMaterialFeatures,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::PipelineKey;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct MaterialDisabledPasses(u8);

impl MaterialDisabledPasses {
    const BASE: u8 = 1 << 0;
    const DEPTH_PREPASS: u8 = 1 << 1;
    const SHADOW: u8 = 1 << 2;
    const VELOCITY: u8 = 1 << 3;
    const TAA_REACTIVE_MASK: u8 = 1 << 4;

    pub(crate) fn from_shader_pass_names(names: &[String]) -> Self {
        let bits = names
            .iter()
            .fold(0_u8, |bits, name| bits | disabled_pass_bit(name.as_str()));
        Self(bits)
    }

    pub(crate) const fn disables_base(self) -> bool {
        (self.0 & Self::BASE) == Self::BASE
    }

    pub(crate) const fn disables_depth_prepass(self) -> bool {
        (self.0 & Self::DEPTH_PREPASS) == Self::DEPTH_PREPASS
    }

    pub(crate) const fn disables_shadow(self) -> bool {
        (self.0 & Self::SHADOW) == Self::SHADOW
    }

    pub(crate) const fn disables_velocity(self) -> bool {
        (self.0 & Self::VELOCITY) == Self::VELOCITY
    }

    pub(crate) const fn disables_taa_reactive_mask(self) -> bool {
        (self.0 & Self::TAA_REACTIVE_MASK) == Self::TAA_REACTIVE_MASK
    }
}

fn disabled_pass_bit(name: &str) -> u8 {
    match name.trim().to_ascii_lowercase().as_str() {
        "base" | "forward" | "gbuffer" | "deferred_gbuffer" => MaterialDisabledPasses::BASE,
        "depth" | "depth_prepass" | "prepass" => MaterialDisabledPasses::DEPTH_PREPASS,
        "shadow" | "shadow_depth" | "shadow_depth_alpha_mask" => MaterialDisabledPasses::SHADOW,
        "velocity" | "motion_vector" | "motion_vectors" => MaterialDisabledPasses::VELOCITY,
        "taa_reactive_mask" | "taa_reactive_material_mask" => {
            MaterialDisabledPasses::TAA_REACTIVE_MASK
        }
        _ => 0,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MaterialCaptureSeed {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    pub(crate) occlusion_strength: f32,
    pub(crate) normal_scale: f32,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_cutoff: Option<f32>,
    pub(crate) lighting_model: RenderMaterialLightingModel,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    pub(crate) cast_shadows: bool,
    pub(crate) receive_shadows: bool,
    pub(crate) disabled_passes: MaterialDisabledPasses,
    pub(crate) taa_reactive_mask_strength: f32,
    pub(crate) subsurface_profile_index: u32,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) base_color_texture_revision: Option<u64>,
    pub(crate) base_color_texture_center_rgba: Option<Vec4>,
    pub(crate) base_color_texture_transform: RenderMaterialTextureTransform,
    pub(crate) base_color_texture_uv_channel: u32,
    pub(crate) normal_texture: Option<ResourceId>,
    pub(crate) normal_texture_revision: Option<u64>,
    pub(crate) normal_texture_center_rgba: Option<Vec4>,
    pub(crate) normal_texture_transform: RenderMaterialTextureTransform,
    pub(crate) normal_texture_uv_channel: u32,
    pub(crate) metallic_roughness_texture: Option<ResourceId>,
    pub(crate) metallic_roughness_texture_revision: Option<u64>,
    pub(crate) metallic_roughness_texture_center_rgba: Option<Vec4>,
    pub(crate) metallic_roughness_texture_transform: RenderMaterialTextureTransform,
    pub(crate) metallic_roughness_texture_uv_channel: u32,
    pub(crate) occlusion_texture: Option<ResourceId>,
    pub(crate) occlusion_texture_revision: Option<u64>,
    pub(crate) occlusion_texture_center_rgba: Option<Vec4>,
    pub(crate) occlusion_texture_transform: RenderMaterialTextureTransform,
    pub(crate) occlusion_texture_uv_channel: u32,
    pub(crate) emissive_texture: Option<ResourceId>,
    pub(crate) emissive_texture_revision: Option<u64>,
    pub(crate) emissive_texture_center_rgba: Option<Vec4>,
    pub(crate) emissive_texture_transform: RenderMaterialTextureTransform,
    pub(crate) emissive_texture_uv_channel: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    pub(crate) occlusion_strength: f32,
    pub(crate) normal_scale: f32,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_cutoff: Option<f32>,
    pub(crate) lighting_model: RenderMaterialLightingModel,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    pub(crate) cast_shadows: bool,
    pub(crate) receive_shadows: bool,
    pub(crate) disabled_passes: MaterialDisabledPasses,
    pub(crate) render_queue: i32,
    pub(crate) render_queue_value: Option<RenderQueueValue>,
    pub(crate) material_queue: i32,
    pub(crate) depth_bias: f32,
    pub(crate) taa_reactive_mask_strength: f32,
    pub(crate) separate_translucency: bool,
    pub(crate) subsurface_profile_index: u32,
    pub(crate) advanced_features: StandardPbrMaterialFeatures,
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
    pub(crate) clearcoat_normal_texture: Option<ResourceId>,
    pub(crate) clearcoat_normal_texture_transform: RenderMaterialTextureTransform,
    pub(crate) clearcoat_normal_texture_uv_channel: u32,
    pub(crate) shader_property_values: BTreeMap<String, RenderMaterialPropertyValue>,
    pub(crate) shader_property_uniform_payload: RenderMaterialPropertyUniformPayload,
    pub(crate) non_standard_texture_slots: BTreeMap<String, Option<ResourceId>>,
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) readiness_report: RenderMaterialReadinessReport,
}

impl MaterialRuntime {
    pub(crate) fn capture_seed(&self) -> MaterialCaptureSeed {
        MaterialCaptureSeed {
            base_color: self.base_color,
            emissive: self.emissive,
            metallic: self.metallic,
            roughness: self.roughness,
            occlusion_strength: self.occlusion_strength,
            normal_scale: self.normal_scale,
            double_sided: self.double_sided,
            alpha_blend: self.alpha_blend,
            alpha_cutoff: self.alpha_cutoff,
            lighting_model: self.lighting_model.clone(),
            shading_model_id: self.shading_model_id,
            unlit: self.unlit,
            cast_shadows: self.cast_shadows,
            receive_shadows: self.receive_shadows,
            disabled_passes: self.disabled_passes,
            taa_reactive_mask_strength: self.taa_reactive_mask_strength,
            subsurface_profile_index: self.subsurface_profile_index,
            base_color_texture: self.base_color_texture,
            base_color_texture_revision: None,
            base_color_texture_center_rgba: None,
            base_color_texture_transform: self.base_color_texture_transform,
            base_color_texture_uv_channel: self.base_color_texture_uv_channel,
            normal_texture: self.normal_texture,
            normal_texture_revision: None,
            normal_texture_center_rgba: None,
            normal_texture_transform: self.normal_texture_transform,
            normal_texture_uv_channel: self.normal_texture_uv_channel,
            metallic_roughness_texture: self.metallic_roughness_texture,
            metallic_roughness_texture_revision: None,
            metallic_roughness_texture_center_rgba: None,
            metallic_roughness_texture_transform: self.metallic_roughness_texture_transform,
            metallic_roughness_texture_uv_channel: self.metallic_roughness_texture_uv_channel,
            occlusion_texture: self.occlusion_texture,
            occlusion_texture_revision: None,
            occlusion_texture_center_rgba: None,
            occlusion_texture_transform: self.occlusion_texture_transform,
            occlusion_texture_uv_channel: self.occlusion_texture_uv_channel,
            emissive_texture: self.emissive_texture,
            emissive_texture_revision: None,
            emissive_texture_center_rgba: None,
            emissive_texture_transform: self.emissive_texture_transform,
            emissive_texture_uv_channel: self.emissive_texture_uv_channel,
        }
    }
}
