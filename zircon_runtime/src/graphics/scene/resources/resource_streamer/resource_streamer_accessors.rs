use crate::asset::assets::AlphaMode;
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::TextureAsset;
use std::sync::Arc;

use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::{GpuModelResource, GpuTextureResource, MaterialCaptureSeed, MaterialRuntime};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn asset_manager(&self) -> Arc<ProjectAssetManager> {
        self.asset_manager.clone()
    }

    pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>> {
        self.models.get(id).map(|prepared| &prepared.resource)
    }

    pub(crate) fn material(&self, id: &ResourceId) -> Option<&MaterialRuntime> {
        self.materials.get(id).map(|prepared| &prepared.runtime)
    }

    pub(crate) fn material_capture_seed(&self, id: &ResourceId) -> Option<MaterialCaptureSeed> {
        self.material(id)
            .map(|material| material.capture_seed())
            .or_else(|| {
                self.asset_manager
                    .load_material_asset(*id)
                    .ok()
                    .map(|material| MaterialCaptureSeed {
                        base_color: Vec4::from_array(material.base_color),
                        emissive: Vec3::from_array(material.emissive),
                        metallic: material.metallic,
                        roughness: material.roughness,
                        double_sided: material.double_sided,
                        alpha_blend: matches!(material.alpha_mode, AlphaMode::Blend),
                        alpha_cutoff: match material.alpha_mode {
                            AlphaMode::Mask { cutoff } => Some(cutoff),
                            _ => None,
                        },
                        base_color_texture: self
                            .resolve_texture_id(material.base_color_texture.as_ref()),
                        normal_texture: self.resolve_texture_id(material.normal_texture.as_ref()),
                        metallic_roughness_texture: self
                            .resolve_texture_id(material.metallic_roughness_texture.as_ref()),
                        occlusion_texture: self
                            .resolve_texture_id(material.occlusion_texture.as_ref()),
                        emissive_texture: self
                            .resolve_texture_id(material.emissive_texture.as_ref()),
                    })
            })
    }

    pub(crate) fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4> {
        id.and_then(|texture_id| {
            self.asset_manager
                .load_texture_asset(texture_id)
                .ok()
                .and_then(|texture| sample_texture_asset_rgba(&texture, uv))
        })
    }

    pub(crate) fn texture(&self, id: Option<ResourceId>) -> Arc<GpuTextureResource> {
        id.and_then(|texture_id| {
            self.textures
                .get(&texture_id)
                .map(|prepared| prepared.resource.clone())
        })
        .unwrap_or_else(|| self.fallback_texture.clone())
    }

    pub(crate) fn shader_source(&self, shader_id: &ResourceId) -> Option<&str> {
        self.shaders
            .get(shader_id)
            .map(|shader| shader.runtime.source.as_str())
    }
}

fn sample_texture_asset_rgba(texture: &TextureAsset, uv: [f32; 2]) -> Option<Vec4> {
    if texture.width == 0 || texture.height == 0 {
        return None;
    }

    let u = wrap01(uv[0]);
    let v = wrap01(uv[1]);
    let x = ((texture.width - 1) as f32 * u).round() as usize;
    let y = ((texture.height - 1) as f32 * v).round() as usize;
    let index = ((y * texture.width as usize) + x) * 4;
    let rgba = texture.rgba.get(index..index + 4)?;
    Some(Vec4::new(
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
        rgba[3] as f32 / 255.0,
    ))
}

fn wrap01(value: f32) -> f32 {
    let wrapped = value.fract();
    if wrapped < 0.0 {
        wrapped + 1.0
    } else {
        wrapped
    }
}
