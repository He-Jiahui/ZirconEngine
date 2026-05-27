use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::TextureAsset;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderMaterialPropertyUniformSummary, RenderMaterialReadinessReport,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::{
    GpuMaterialUniformResource, GpuModelResource, GpuTextureResource, MaterialCaptureSeed,
    MaterialRuntime,
};
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

    pub(crate) fn material_uniform(&self, id: &ResourceId) -> Arc<GpuMaterialUniformResource> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.clone())
            .unwrap_or_else(|| self.fallback_material_uniform.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_payload_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.payload_byte_len)
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_buffer_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.buffer_byte_len)
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_field_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .layout
                .len()
        })
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_unsupported_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .unsupported
                .len()
        })
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyUniformSummary> {
        self.materials
            .get(id)
            .map(|prepared| prepared.runtime.shader_property_uniform_payload.summary())
    }

    #[allow(dead_code)]
    pub(crate) fn material_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<&RenderMaterialReadinessReport> {
        self.material(id).map(|material| &material.readiness_report)
    }

    #[allow(dead_code)]
    pub(crate) fn material_capture_seed(&self, id: &ResourceId) -> Option<MaterialCaptureSeed> {
        self.material(id)
            .map(|material| material.capture_seed())
            .or_else(|| {
                self.asset_manager
                    .load_material_asset(*id)
                    .ok()
                    .map(|material| {
                        let descriptor = material.standard_material_descriptor();
                        MaterialCaptureSeed {
                            base_color: Vec4::from_array(descriptor.base_color),
                            emissive: Vec3::from_array(descriptor.emissive),
                            metallic: descriptor.metallic,
                            roughness: descriptor.roughness,
                            double_sided: descriptor.double_sided,
                            alpha_blend: matches!(
                                descriptor.alpha_mode,
                                RenderMaterialAlphaMode::Blend
                            ),
                            alpha_cutoff: match descriptor.alpha_mode {
                                RenderMaterialAlphaMode::Mask { cutoff } => Some(cutoff),
                                _ => None,
                            },
                            unlit: descriptor.unlit,
                            base_color_texture: self
                                .resolve_texture_reference(
                                    "base_color_texture",
                                    descriptor.base_color_texture.as_ref(),
                                )
                                .id(),
                            normal_texture: self
                                .resolve_texture_reference(
                                    "normal_texture",
                                    descriptor.normal_texture.as_ref(),
                                )
                                .id(),
                            metallic_roughness_texture: self
                                .resolve_texture_reference(
                                    "metallic_roughness_texture",
                                    descriptor.metallic_roughness_texture.as_ref(),
                                )
                                .id(),
                            occlusion_texture: self
                                .resolve_texture_reference(
                                    "occlusion_texture",
                                    descriptor.occlusion_texture.as_ref(),
                                )
                                .id(),
                            emissive_texture: self
                                .resolve_texture_reference(
                                    "emissive_texture",
                                    descriptor.emissive_texture.as_ref(),
                                )
                                .id(),
                        }
                    })
            })
    }

    #[allow(dead_code)]
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

    pub(crate) fn last_material_count(&self) -> usize {
        self.last_material_count
    }

    pub(crate) fn last_material_ready_count(&self) -> usize {
        self.last_material_ready_count
    }

    pub(crate) fn last_material_fallback_count(&self) -> usize {
        self.last_material_fallback_count
    }

    pub(crate) fn last_material_validation_error_count(&self) -> usize {
        self.last_material_validation_error_count
    }

    pub(crate) fn last_material_diagnostic_count(&self) -> usize {
        self.last_material_diagnostic_count
    }

    pub(crate) fn last_sprite_count(&self) -> usize {
        self.last_sprite_count
    }

    pub(crate) fn last_sprite_ready_count(&self) -> usize {
        self.last_sprite_ready_count
    }

    pub(crate) fn last_sprite_texture_fallback_count(&self) -> usize {
        self.last_sprite_texture_fallback_count
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn wrap01(value: f32) -> f32 {
    let wrapped = value.fract();
    if wrapped < 0.0 {
        wrapped + 1.0
    } else {
        wrapped
    }
}
