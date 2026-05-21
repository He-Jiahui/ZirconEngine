use crate::asset::{AssetReference, ShaderAsset};
use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialValidationError,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, ResourceLocator};

use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedMaterial;
use super::super::{texture_upload_support_from_device, MaterialRuntime, PipelineKey};
use super::ResourceStreamer;

const FALLBACK_MATERIAL_URI: &str = "builtin://missing-material";

impl ResourceStreamer {
    pub(crate) fn ensure_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let (material, missing_material_fallback) = match self.asset_manager.load_material_asset(id)
        {
            Ok(material) => (material, None),
            Err(error) => {
                let fallback_uri = fallback_material_uri();
                let fallback_id = self.asset_manager.resolve_asset_id(&fallback_uri).ok_or_else(
                    || {
                        GraphicsError::Asset(format!(
                            "missing material {id} ({error}); fallback material {fallback_uri} is not registered"
                        ))
                    },
                )?;
                let material = self.asset_manager.load_material_asset(fallback_id).map_err(
                    |fallback_error| {
                        GraphicsError::Asset(format!(
                            "missing material {id} ({error}); fallback material {fallback_uri} failed to load: {fallback_error}"
                        ))
                    },
                )?;
                (material, Some(missing_material_fallback_usage(id)))
            }
        };
        let shader_contract = self.load_shader_contract(material.shader.clone());
        let descriptor = shader_contract
            .as_ref()
            .map(|shader| material.standard_material_descriptor_for_shader(shader))
            .unwrap_or_else(|| material.standard_material_descriptor());
        let shader_resolver = self.asset_manager.clone();
        let texture_resolver = self.asset_manager.clone();
        let mut readiness = if let Some(shader) = shader_contract.as_ref() {
            material.readiness_report_with_shader_contract(
                shader,
                move |reference| {
                    shader_resolver
                        .resolve_asset_id(&reference.locator)
                        .is_some()
                },
                move |reference| {
                    texture_resolver
                        .resolve_asset_id(&reference.locator)
                        .is_some()
                },
            )
        } else {
            let shader_resolver = self.asset_manager.clone();
            let texture_resolver = self.asset_manager.clone();
            material.readiness_report_with_resolution(
                move |reference| {
                    shader_resolver
                        .resolve_asset_id(&reference.locator)
                        .is_some()
                },
                move |reference| {
                    texture_resolver
                        .resolve_asset_id(&reference.locator)
                        .is_some()
                },
            )
        };
        if let Some((validation_error, fallback_usage)) = missing_material_fallback {
            readiness.push_validation_error_once(validation_error);
            readiness.push_fallback_usage_once(fallback_usage);
        }
        let (alpha_blend, alpha_mask, alpha_cutoff) = match descriptor.alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false, None),
            RenderMaterialAlphaMode::Mask { cutoff } => (false, true, Some(cutoff)),
            RenderMaterialAlphaMode::Blend => (true, false, None),
        };
        let texture_support = texture_upload_support_from_device(device);
        let base_color_texture = self.resolve_texture_reference_with_support(
            "base_color_texture",
            descriptor.base_color_texture.as_ref(),
            texture_support,
        );
        let normal_texture = self.resolve_texture_reference_with_support(
            "normal_texture",
            descriptor.normal_texture.as_ref(),
            texture_support,
        );
        let metallic_roughness_texture = self.resolve_texture_reference_with_support(
            "metallic_roughness_texture",
            descriptor.metallic_roughness_texture.as_ref(),
            texture_support,
        );
        let occlusion_texture = self.resolve_texture_reference_with_support(
            "occlusion_texture",
            descriptor.occlusion_texture.as_ref(),
            texture_support,
        );
        let emissive_texture = self.resolve_texture_reference_with_support(
            "emissive_texture",
            descriptor.emissive_texture.as_ref(),
            texture_support,
        );
        let shader_slot_textures = material
            .all_texture_slots()
            .into_iter()
            .filter(|(slot, _)| !is_standard_texture_slot(slot))
            .map(|(slot, texture)| {
                self.resolve_texture_reference_with_support(&slot, Some(texture), texture_support)
            })
            .collect::<Vec<_>>();
        for texture in [
            &base_color_texture,
            &normal_texture,
            &metallic_roughness_texture,
            &occlusion_texture,
            &emissive_texture,
        ] {
            if let Some(error) = &texture.validation_error {
                readiness.push_validation_error_once(error.clone());
            }
            if let Some(usage) = &texture.fallback_usage {
                readiness.push_fallback_usage_once(usage.clone());
            }
        }
        for texture in &shader_slot_textures {
            if let Some(error) = &texture.validation_error {
                readiness.push_validation_error_once(error.clone());
            }
            if let Some(usage) = &texture.fallback_usage {
                readiness.push_fallback_usage_once(usage.clone());
            }
        }
        let (shader_id, shader_revision, shader_readiness) =
            self.ensure_shader_source(&descriptor.dependencies.shader)?;
        if let Some(shader_readiness) = shader_readiness {
            for error in shader_readiness.validation_errors {
                readiness.push_validation_error_once(error);
            }
            for usage in shader_readiness.fallback_usages {
                readiness.push_fallback_usage_once(usage);
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
            base_color: Vec4::from_array(descriptor.base_color),
            emissive: Vec3::from_array(descriptor.emissive),
            metallic: descriptor.metallic,
            roughness: descriptor.roughness,
            double_sided: descriptor.double_sided,
            alpha_blend,
            alpha_cutoff,
            unlit: descriptor.unlit,
            base_color_texture: base_color_texture.id(),
            normal_texture: normal_texture.id(),
            metallic_roughness_texture: metallic_roughness_texture.id(),
            occlusion_texture: occlusion_texture.id(),
            emissive_texture: emissive_texture.id(),
            pipeline_key: PipelineKey {
                shader_id,
                shader_revision,
                double_sided: descriptor.double_sided,
                alpha_blend,
                alpha_mask,
                alpha_cutoff_bits: alpha_cutoff.map(f32::to_bits),
                unlit: descriptor.unlit,
                has_base_color_texture: descriptor.base_color_texture.is_some(),
                has_normal_texture: descriptor.normal_texture.is_some(),
                has_metallic_roughness_texture: descriptor.metallic_roughness_texture.is_some(),
                has_occlusion_texture: descriptor.occlusion_texture.is_some(),
                has_emissive_texture: descriptor.emissive_texture.is_some(),
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
        .chain(
            shader_slot_textures
                .iter()
                .filter_map(|texture| texture.id()),
        ) {
            self.ensure_texture(device, queue, texture_layout, texture_id)?;
        }
        self.materials.insert(id, PreparedMaterial { runtime });
        Ok(())
    }

    fn load_shader_contract(&self, reference: AssetReference) -> Option<ShaderAsset> {
        self.asset_manager
            .resolve_asset_id(&reference.locator)
            .and_then(|id| self.asset_manager.load_shader_asset(id).ok())
    }
}

fn fallback_material_uri() -> ResourceLocator {
    ResourceLocator::parse(FALLBACK_MATERIAL_URI).expect("builtin fallback material uri")
}

fn missing_material_fallback_usage(
    material: ResourceId,
) -> (RenderMaterialValidationError, RenderMaterialFallbackUsage) {
    (
        RenderMaterialValidationError::UnresolvedMaterialReference { material },
        RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Material { material },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        },
    )
}

fn is_standard_texture_slot(slot: &str) -> bool {
    matches!(
        slot,
        "base_color"
            | "base_color_texture"
            | "normal"
            | "normal_texture"
            | "metallic_roughness"
            | "metallic_roughness_texture"
            | "occlusion"
            | "occlusion_texture"
            | "emissive"
            | "emissive_texture"
    )
}
