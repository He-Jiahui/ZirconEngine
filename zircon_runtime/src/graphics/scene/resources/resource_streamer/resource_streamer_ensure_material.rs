use crate::asset::assets::AlphaMode;
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle};

use crate::graphics::types::GraphicsError;

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
        let (alpha_blend, alpha_cutoff) = match &material.alpha_mode {
            AlphaMode::Opaque => (false, None),
            AlphaMode::Mask { cutoff } => (false, Some(*cutoff)),
            AlphaMode::Blend => (true, None),
        };
        let base_color_texture = self.resolve_texture_id(material.base_color_texture.as_ref());
        let normal_texture = self.resolve_texture_id(material.normal_texture.as_ref());
        let metallic_roughness_texture =
            self.resolve_texture_id(material.metallic_roughness_texture.as_ref());
        let occlusion_texture = self.resolve_texture_id(material.occlusion_texture.as_ref());
        let emissive_texture = self.resolve_texture_id(material.emissive_texture.as_ref());
        for texture_id in [
            base_color_texture,
            normal_texture,
            metallic_roughness_texture,
            occlusion_texture,
            emissive_texture,
        ]
        .into_iter()
        .flatten()
        {
            self.ensure_texture(device, queue, texture_layout, texture_id)?;
        }
        let (shader_id, shader_revision) = self.ensure_shader_source(&material.shader)?;
        self.materials.insert(
            id,
            PreparedMaterial {
                revision,
                runtime: MaterialRuntime {
                    base_color: Vec4::from_array(material.base_color),
                    emissive: Vec3::from_array(material.emissive),
                    metallic: material.metallic,
                    roughness: material.roughness,
                    double_sided: material.double_sided,
                    alpha_blend,
                    alpha_cutoff,
                    base_color_texture,
                    normal_texture,
                    metallic_roughness_texture,
                    occlusion_texture,
                    emissive_texture,
                    pipeline_key: PipelineKey {
                        shader_id,
                        shader_revision,
                        double_sided: material.double_sided,
                        alpha_blend,
                    },
                },
            },
        );
        Ok(())
    }
}
