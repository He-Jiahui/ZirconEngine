use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::PipelineKey;

#[derive(Clone, Copy, Debug)]
pub(crate) struct MaterialCaptureSeed {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_cutoff: Option<f32>,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) normal_texture: Option<ResourceId>,
    pub(crate) metallic_roughness_texture: Option<ResourceId>,
    pub(crate) occlusion_texture: Option<ResourceId>,
    pub(crate) emissive_texture: Option<ResourceId>,
}

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    pub(crate) base_color: Vec4,
    pub(crate) emissive: Vec3,
    pub(crate) metallic: f32,
    pub(crate) roughness: f32,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_cutoff: Option<f32>,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) normal_texture: Option<ResourceId>,
    pub(crate) metallic_roughness_texture: Option<ResourceId>,
    pub(crate) occlusion_texture: Option<ResourceId>,
    pub(crate) emissive_texture: Option<ResourceId>,
    pub(crate) pipeline_key: PipelineKey,
}

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
            base_color_texture: self.base_color_texture,
            normal_texture: self.normal_texture,
            metallic_roughness_texture: self.metallic_roughness_texture,
            occlusion_texture: self.occlusion_texture,
            emissive_texture: self.emissive_texture,
        }
    }
}
