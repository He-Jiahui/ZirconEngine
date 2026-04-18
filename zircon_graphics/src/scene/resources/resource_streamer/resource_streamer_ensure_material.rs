use zircon_asset::AlphaMode;
use zircon_math::Vec4;
use zircon_resource::{MaterialMarker, ResourceHandle};

use crate::types::GraphicsError;

use super::super::prepared::PreparedMaterial;
use super::super::{MaterialRuntime, PipelineKey};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let revision = self.resource_revision(id)?;
        if self
            .materials
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let material = self
            .asset_manager
            .load_material_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let texture_id = self.resolve_texture_id(&material);
        if let Some(texture_id) = texture_id {
            self.ensure_texture(device, queue, texture_layout, texture_id)?;
        }
        let (shader_id, shader_revision) = self.ensure_shader_source(&material.shader)?;
        self.materials.insert(
            id,
            PreparedMaterial {
                revision,
                runtime: MaterialRuntime {
                    base_color: Vec4::from_array(material.base_color),
                    base_color_texture: texture_id,
                    pipeline_key: PipelineKey {
                        shader_id,
                        shader_revision,
                        double_sided: material.double_sided,
                        alpha_blend: matches!(material.alpha_mode, AlphaMode::Blend),
                    },
                },
            },
        );
        Ok(())
    }
}
