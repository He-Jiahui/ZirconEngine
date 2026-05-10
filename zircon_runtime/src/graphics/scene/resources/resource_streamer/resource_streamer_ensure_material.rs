use crate::asset::assets::AlphaMode;
use crate::core::framework::render::RenderMaterialValidationError;
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
        self.resource_revision(id)?;
        let material = self
            .asset_manager
            .load_material_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let mut readiness = material.readiness_report();
        let (alpha_blend, alpha_cutoff) = match &material.alpha_mode {
            AlphaMode::Opaque => (false, None),
            AlphaMode::Mask { cutoff } => (false, Some(*cutoff)),
            AlphaMode::Blend => (true, None),
        };
        let base_color_texture = self
            .resolve_texture_reference("base_color_texture", material.base_color_texture.as_ref());
        let normal_texture =
            self.resolve_texture_reference("normal_texture", material.normal_texture.as_ref());
        let metallic_roughness_texture = self.resolve_texture_reference(
            "metallic_roughness_texture",
            material.metallic_roughness_texture.as_ref(),
        );
        let occlusion_texture = self
            .resolve_texture_reference("occlusion_texture", material.occlusion_texture.as_ref());
        let emissive_texture =
            self.resolve_texture_reference("emissive_texture", material.emissive_texture.as_ref());
        for texture in [
            &base_color_texture,
            &normal_texture,
            &metallic_roughness_texture,
            &occlusion_texture,
            &emissive_texture,
        ] {
            if let Some(error) = &texture.validation_error {
                if !readiness.validation_errors.contains(error) {
                    readiness.validation_errors.push(error.clone());
                }
            }
            if let Some(usage) = &texture.fallback_usage {
                if !readiness.fallback_usages.contains(usage) {
                    readiness.fallback_usages.push(usage.clone());
                }
            }
        }
        let (shader_id, shader_revision, shader_readiness) =
            self.ensure_shader_source(&material.shader)?;
        if let Some(shader_readiness) = shader_readiness {
            for error in shader_readiness.validation_errors {
                if !readiness.validation_errors.contains(&error) {
                    readiness.validation_errors.push(error);
                }
            }
            for usage in shader_readiness.fallback_usages {
                if !readiness.fallback_usages.contains(&usage) {
                    readiness.fallback_usages.push(usage);
                }
            }
        }
        let has_blocking_validation = readiness.validation_errors.iter().any(|error| {
            matches!(
                error,
                RenderMaterialValidationError::InvalidMaskCutoff { .. }
                    | RenderMaterialValidationError::MissingRuntimeShaderSource
            )
        });
        let runtime = MaterialRuntime {
            base_color: Vec4::from_array(material.base_color),
            emissive: Vec3::from_array(material.emissive),
            metallic: material.metallic,
            roughness: material.roughness,
            double_sided: material.double_sided,
            alpha_blend,
            alpha_cutoff,
            base_color_texture: base_color_texture.id(),
            normal_texture: normal_texture.id(),
            metallic_roughness_texture: metallic_roughness_texture.id(),
            occlusion_texture: occlusion_texture.id(),
            emissive_texture: emissive_texture.id(),
            pipeline_key: PipelineKey {
                shader_id,
                shader_revision,
                double_sided: material.double_sided,
                alpha_blend,
            },
            readiness_report: readiness,
        };
        if has_blocking_validation {
            let validation_errors = runtime.readiness_report.validation_errors.clone();
            self.materials.insert(id, PreparedMaterial { runtime });
            return Err(GraphicsError::Asset(format!(
                "material {} is not render-ready: {:?}",
                id, validation_errors
            )));
        }
        for texture_id in [
            base_color_texture.id(),
            normal_texture.id(),
            metallic_roughness_texture.id(),
            occlusion_texture.id(),
            emissive_texture.id(),
        ]
        .into_iter()
        .flatten()
        {
            self.ensure_texture(device, queue, texture_layout, texture_id)?;
        }
        self.materials.insert(id, PreparedMaterial { runtime });
        Ok(())
    }
}
