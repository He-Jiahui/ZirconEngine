use std::sync::Arc;

use crate::core::framework::render::{RenderFrameSubmissionTransaction, RenderImageUsage};
use crate::core::resource::ResourceId;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::super::super::MaterialRuntime;
use super::super::super::prepared::{
    PreparedMaterialTextureBinding, PreparedMaterialTextureResource, PreparedMaterialTextureSet,
};
use super::super::ResourceStreamer;

impl ResourceStreamer {
    pub(super) fn prepared_material_texture_set(
        &self,
        runtime: &MaterialRuntime,
    ) -> PreparedMaterialTextureSet {
        PreparedMaterialTextureSet {
            base_color: self.prepared_material_texture_binding(runtime.base_color_texture, false),
            normal: self.prepared_material_texture_binding(runtime.normal_texture, true),
            metallic_roughness: self
                .prepared_material_texture_binding(runtime.metallic_roughness_texture, false),
            occlusion: self.prepared_material_texture_binding(runtime.occlusion_texture, false),
            emissive: self.prepared_material_texture_binding(runtime.emissive_texture, false),
            clearcoat_normal: self
                .prepared_material_texture_binding(runtime.clearcoat_normal_texture, true),
        }
    }

    fn prepared_material_texture_binding(
        &self,
        id: Option<ResourceId>,
        normal_fallback: bool,
    ) -> PreparedMaterialTextureBinding {
        if let Some((id, prepared)) = id.and_then(|id| {
            self.output_target_textures
                .get(&id)
                .filter(|prepared| {
                    prepared
                        .resource()
                        .descriptor()
                        .usage
                        .contains(&RenderImageUsage::Sampled)
                })
                .map(|prepared| (id, prepared))
        }) {
            return PreparedMaterialTextureBinding {
                id: Some(id),
                revision: Some(prepared.revision),
                capture_sample_rgba: None,
                resource: PreparedMaterialTextureResource::OutputTarget(Arc::clone(
                    prepared.resource(),
                )),
            };
        }
        if let Some((id, prepared)) =
            id.and_then(|id| self.textures.get(&id).map(|prepared| (id, prepared)))
        {
            return PreparedMaterialTextureBinding {
                id: Some(id),
                revision: Some(prepared.revision),
                capture_sample_rgba: prepared.capture_sample_rgba,
                resource: PreparedMaterialTextureResource::Texture(Arc::clone(&prepared.resource)),
            };
        }
        let fallback = if normal_fallback {
            &self.fallback_normal_texture
        } else {
            &self.fallback_texture
        };
        PreparedMaterialTextureBinding {
            id,
            revision: None,
            capture_sample_rgba: None,
            resource: PreparedMaterialTextureResource::Texture(Arc::clone(fallback)),
        }
    }

    pub(super) fn ensure_material_texture(
        &mut self,
        backend: &RenderBackend,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        texture_id: ResourceId,
        submission_transaction: Option<&mut RenderFrameSubmissionTransaction>,
    ) -> Result<(), GraphicsError> {
        if self.material_texture_uses_output_target_binding(texture_id) {
            self.ensure_output_target_texture_resource(device, texture_id)
        } else {
            match submission_transaction {
                Some(transaction) => {
                    self.ensure_texture_for_frame(backend, texture_layout, texture_id, transaction)
                }
                None => self.ensure_texture(backend, texture_layout, texture_id),
            }
        }
    }

    fn material_texture_uses_output_target_binding(&self, texture_id: ResourceId) -> bool {
        self.asset_manager()
            .ok()
            .and_then(|asset_manager| asset_manager.load_texture_asset(texture_id).ok())
            .map(|texture| {
                let descriptor = texture.render_image_descriptor();
                descriptor.usage.contains(&RenderImageUsage::RenderTarget)
                    && descriptor.usage.contains(&RenderImageUsage::Sampled)
            })
            .unwrap_or(false)
    }
}
