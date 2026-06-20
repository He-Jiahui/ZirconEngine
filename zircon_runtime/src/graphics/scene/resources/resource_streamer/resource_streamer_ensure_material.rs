use std::collections::BTreeMap;

use crate::asset::{AssetReference, ShaderAsset};
use crate::core::framework::render::{
    RenderImageUsage, RenderMaterialAlphaMode, RenderMaterialDiagnosticSource,
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialLightingModel, RenderMaterialPropertyUniformPayload,
    RenderMaterialPropertyValueState, RenderMaterialPropertyValueSummary,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialValidationError, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, ResourceLocator};

use crate::graphics::types::GraphicsError;

use crate::graphics::material::builtin_shading_model_registry;

use super::super::prepared::PreparedMaterial;
use super::super::{
    default_pipeline_key, texture_upload_support_from_device, GpuMaterialUniformResource,
    MaterialRuntime, PipelineKey,
};
use super::resource_streamer_validate_material_shader_layout::renderer_material_layout_diagnostics;
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
        let requested_revision = self.resource_revision(id).ok();
        if let Some(prepared) = self
            .materials
            .get(&id)
            .filter(|prepared| prepared.revision == requested_revision)
        {
            return material_prepare_result(id, &prepared.runtime.readiness_report);
        }
        let (material, missing_material_fallback, prepared_revision) = match self
            .asset_manager
            .load_material_asset(id)
        {
            Ok(material) => (material, None, requested_revision),
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
                (material, Some(missing_material_fallback_usage(id)), None)
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
        let uses_renderer_material_abi_fallback = if let Some(shader) = shader_contract.as_ref() {
            let abi_diagnostics = renderer_material_layout_diagnostics(shader);
            let uses_fallback = !abi_diagnostics.is_empty();
            for error in abi_diagnostics {
                readiness.push_validation_error_once(error);
            }
            uses_fallback
        } else {
            false
        };
        if uses_renderer_material_abi_fallback {
            readiness.push_fallback_usage_once(RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Validation,
                fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            });
        }
        let (alpha_blend, alpha_mask, alpha_cutoff) = match descriptor.alpha_mode {
            RenderMaterialAlphaMode::Opaque => (false, false, None),
            RenderMaterialAlphaMode::Mask { cutoff } => (false, true, Some(cutoff)),
            RenderMaterialAlphaMode::Blend => (true, false, None),
        };
        let lighting_model = if descriptor.unlit {
            RenderMaterialLightingModel::Unlit
        } else {
            descriptor.lighting_model.clone()
        };
        let shading_model_registry = builtin_shading_model_registry();
        let shading_model_descriptor =
            shading_model_registry.resolve_lighting_model(&lighting_model);
        let shading_model_id = shading_model_descriptor
            .map(|descriptor| descriptor.id)
            .unwrap_or(SHADING_MODEL_ID_STANDARD_PBR);
        if shading_model_descriptor.is_none() {
            readiness.push_validation_error_once(
                RenderMaterialValidationError::UnregisteredShadingModel {
                    path: "overrides.lighting_model".to_string(),
                    token: lighting_model.as_token(),
                },
            );
            readiness.push_fallback_usage_once(RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Validation,
                fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            });
        }
        let unlit = lighting_model.is_unlit();
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
        let standard_texture_slots = [
            descriptor.base_color_texture.as_ref().map(|_| {
                (
                    "base_color",
                    base_color_texture.id(),
                    base_color_texture.slot_fallback.clone(),
                )
            }),
            descriptor.normal_texture.as_ref().map(|_| {
                (
                    "normal",
                    normal_texture.id(),
                    normal_texture.slot_fallback.clone(),
                )
            }),
            descriptor.metallic_roughness_texture.as_ref().map(|_| {
                (
                    "metallic_roughness",
                    metallic_roughness_texture.id(),
                    metallic_roughness_texture.slot_fallback.clone(),
                )
            }),
            descriptor.occlusion_texture.as_ref().map(|_| {
                (
                    "occlusion",
                    occlusion_texture.id(),
                    occlusion_texture.slot_fallback.clone(),
                )
            }),
            descriptor.emissive_texture.as_ref().map(|_| {
                (
                    "emissive",
                    emissive_texture.id(),
                    emissive_texture.slot_fallback.clone(),
                )
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let standard_texture_slot_ids = standard_texture_slots
            .iter()
            .map(|(_, texture_id, _)| *texture_id)
            .collect::<Vec<_>>();
        let shader_slot_textures = material
            .all_texture_slots()
            .into_iter()
            .filter(|(slot, _)| !is_standard_texture_slot(slot))
            .map(|(slot, texture)| {
                let resolved = self.resolve_texture_reference_with_support(
                    &slot,
                    Some(texture),
                    texture_support,
                );
                (slot, resolved)
            })
            .collect::<Vec<_>>();
        let shader_property_values = shader_contract
            .as_ref()
            .map(|shader| material.shader_property_values_for_shader(shader))
            .unwrap_or_default();
        let shader_property_value_summary =
            RenderMaterialPropertyValueSummary::from_values(&shader_property_values);
        let shader_property_value_states =
            RenderMaterialPropertyValueState::from_values(&shader_property_values);
        let shader_property_uniform_payload =
            RenderMaterialPropertyUniformPayload::from_values(&shader_property_values);
        for diagnostic in shader_property_uniform_payload.unsupported_diagnostics() {
            readiness.push_diagnostic_once(diagnostic);
        }
        readiness.property_value_summary = Some(shader_property_value_summary);
        readiness.property_value_states = shader_property_value_states;
        readiness.uniform_summary = Some(shader_property_uniform_payload.summary());
        readiness.uniform_fields = shader_property_uniform_payload.layout.clone();
        readiness.uniform_unsupported = shader_property_uniform_payload.unsupported.clone();
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
        for (_slot, texture) in &shader_slot_textures {
            if let Some(error) = &texture.validation_error {
                readiness.push_validation_error_once(error.clone());
            }
            if let Some(usage) = &texture.fallback_usage {
                readiness.push_fallback_usage_once(usage.clone());
            }
        }
        let non_standard_texture_slots = shader_slot_textures
            .iter()
            .map(|(slot, texture)| (slot.clone(), texture.id()))
            .collect::<BTreeMap<_, _>>();
        readiness.standard_texture_slot_summary = Some(
            RenderMaterialTextureSlotSummary::from_texture_ids(&standard_texture_slot_ids),
        );
        readiness.standard_texture_slot_states =
            RenderMaterialTextureSlotState::from_resolved_slots(
                standard_texture_slots
                    .iter()
                    .map(|(slot, texture_id, fallback)| (*slot, *texture_id, fallback.clone())),
            );
        readiness.texture_slot_summary = Some(
            RenderMaterialTextureSlotSummary::from_non_standard_slots(&non_standard_texture_slots),
        );
        readiness.non_standard_texture_slot_states =
            RenderMaterialTextureSlotState::from_resolved_slots(shader_slot_textures.iter().map(
                |(slot, texture)| (slot.clone(), texture.id(), texture.slot_fallback.clone()),
            ));
        let (shader_id, shader_revision, shader_readiness) =
            self.ensure_shader_source(&descriptor.dependencies.shader)?;
        if let Some(shader_readiness) = shader_readiness {
            for error in shader_readiness.validation_errors {
                readiness.push_validation_error_once(error);
            }
            for usage in shader_readiness.fallback_usages {
                readiness.push_fallback_usage_once(usage);
            }
            for diagnostic in shader_readiness.diagnostics {
                readiness.push_diagnostic_once(diagnostic);
            }
        }
        let (pipeline_shader_id, pipeline_shader_revision) =
            if material_uses_renderer_material_abi_fallback(&readiness.validation_errors) {
                let fallback_key = default_pipeline_key();
                (fallback_key.shader_id, fallback_key.shader_revision)
            } else {
                (shader_id, shader_revision)
            };
        let runtime = MaterialRuntime {
            base_color: Vec4::from_array(descriptor.base_color),
            emissive: Vec3::from_array(descriptor.emissive),
            metallic: descriptor.metallic,
            roughness: descriptor.roughness,
            double_sided: descriptor.double_sided,
            alpha_blend,
            alpha_cutoff,
            lighting_model: lighting_model.clone(),
            shading_model_id,
            unlit,
            cast_shadows: descriptor.cast_shadows,
            receive_shadows: descriptor.receive_shadows,
            render_queue: descriptor.render_queue,
            render_queue_value: descriptor.render_queue_value,
            material_queue: descriptor.material_queue,
            depth_bias: descriptor.depth_bias,
            taa_reactive_mask_strength: descriptor.taa_reactive_mask_strength,
            base_color_texture: base_color_texture.id(),
            base_color_texture_transform: descriptor.base_color_texture_transform,
            base_color_texture_uv_channel: descriptor.base_color_texture_uv_channel,
            normal_texture: normal_texture.id(),
            normal_texture_transform: descriptor.normal_texture_transform,
            normal_texture_uv_channel: descriptor.normal_texture_uv_channel,
            metallic_roughness_texture: metallic_roughness_texture.id(),
            metallic_roughness_texture_transform: descriptor.metallic_roughness_texture_transform,
            metallic_roughness_texture_uv_channel: descriptor.metallic_roughness_texture_uv_channel,
            occlusion_texture: occlusion_texture.id(),
            occlusion_texture_transform: descriptor.occlusion_texture_transform,
            occlusion_texture_uv_channel: descriptor.occlusion_texture_uv_channel,
            emissive_texture: emissive_texture.id(),
            emissive_texture_transform: descriptor.emissive_texture_transform,
            emissive_texture_uv_channel: descriptor.emissive_texture_uv_channel,
            shader_property_values,
            shader_property_uniform_payload,
            non_standard_texture_slots,
            pipeline_key: PipelineKey {
                shader_id: pipeline_shader_id,
                shader_revision: pipeline_shader_revision,
                double_sided: descriptor.double_sided,
                alpha_blend,
                alpha_mask,
                alpha_cutoff_bits: alpha_cutoff.map(f32::to_bits),
                shading_model_id,
                unlit,
                has_base_color_texture: descriptor.base_color_texture.is_some(),
                has_normal_texture: descriptor.normal_texture.is_some(),
                has_metallic_roughness_texture: descriptor.metallic_roughness_texture.is_some(),
                has_occlusion_texture: descriptor.occlusion_texture.is_some(),
                has_emissive_texture: descriptor.emissive_texture.is_some(),
            },
            readiness_report: readiness,
        };
        let uniform = std::sync::Arc::new(GpuMaterialUniformResource::from_payload(
            device,
            &runtime.shader_property_uniform_payload,
        ));
        let standard_uniform = std::sync::Arc::new(
            GpuMaterialUniformResource::from_standard_material(device, &runtime),
        );
        let prepare_result = material_prepare_result(id, &runtime.readiness_report);
        if prepare_result.is_err() {
            self.materials.insert(
                id,
                PreparedMaterial {
                    revision: prepared_revision,
                    runtime,
                    uniform,
                    standard_uniform,
                },
            );
            return prepare_result;
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
                .filter_map(|(_slot, texture)| texture.id()),
        ) {
            self.ensure_material_texture(device, queue, texture_layout, texture_id)?;
        }
        self.materials.insert(
            id,
            PreparedMaterial {
                revision: prepared_revision,
                runtime,
                uniform,
                standard_uniform,
            },
        );
        Ok(())
    }

    fn ensure_material_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        texture_id: ResourceId,
    ) -> Result<(), GraphicsError> {
        if self.material_texture_uses_output_target_binding(texture_id) {
            self.ensure_output_target_texture_resource(device, texture_id)
        } else {
            self.ensure_texture(device, queue, texture_layout, texture_id)
        }
    }

    fn material_texture_uses_output_target_binding(&self, texture_id: ResourceId) -> bool {
        self.asset_manager
            .load_texture_asset(texture_id)
            .ok()
            .map(|texture| {
                let descriptor = texture.render_image_descriptor();
                descriptor.usage.contains(&RenderImageUsage::RenderTarget)
                    && descriptor.usage.contains(&RenderImageUsage::Sampled)
            })
            .unwrap_or(false)
    }

    fn load_shader_contract(&self, reference: AssetReference) -> Option<ShaderAsset> {
        self.asset_manager
            .resolve_asset_id(&reference.locator)
            .and_then(|id| self.asset_manager.load_shader_asset(id).ok())
    }
}

fn material_prepare_result(
    id: ResourceId,
    report: &crate::core::framework::render::RenderMaterialReadinessReport,
) -> Result<(), GraphicsError> {
    if has_blocking_material_validation(&report.validation_errors) {
        Err(GraphicsError::Asset(format!(
            "material {} is not render-ready: {:?}",
            id, report.validation_errors
        )))
    } else {
        Ok(())
    }
}

fn has_blocking_material_validation(validation_errors: &[RenderMaterialValidationError]) -> bool {
    validation_errors.iter().any(|error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { .. }
                | RenderMaterialValidationError::MissingRuntimeShaderSource
        )
    })
}

fn material_uses_renderer_material_abi_fallback(
    validation_errors: &[RenderMaterialValidationError],
) -> bool {
    validation_errors.iter().any(|error| {
        matches!(
            error,
            RenderMaterialValidationError::ShaderReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::RendererMaterialAbi,
                ..
            }
        )
    })
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
            | "albedo"
            | "diffuse"
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
