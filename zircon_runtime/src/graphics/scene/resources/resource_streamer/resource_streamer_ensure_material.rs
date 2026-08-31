use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::asset::{AssetReference, TextureUploadSupport};
use crate::core::framework::render::{
    RenderFrameSubmissionTransaction, RenderMaterialAlphaMode, RenderMaterialFallbackPolicy,
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialLightingModel,
    RenderMaterialPropertyUniformPayload, RenderMaterialPropertyValueState,
    RenderMaterialPropertyValueSummary, RenderMaterialTextureDimension,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialValidationError, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, ResourceLocator};

use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::super::prepared::{
    PreparedMaterial, PreparedMaterialBundle, PreparedMaterialCandidateIdentity,
    PreparedMaterialDependency, PreparedMaterialShaderDependency,
    PreparedMaterialTextureDependency,
};
use super::super::{
    GpuMaterialUniformResource, MaterialDisabledPasses, MaterialRuntime, PipelineKey,
    default_pipeline_key, texture_upload_support_from_device,
};
use super::ResourceStreamer;
use super::resource_streamer_validate_material_shader_layout::renderer_material_layout_diagnostics;

mod cache_identity;
mod candidate_publication;
mod material_readiness;
mod shader_contract_snapshot;
#[cfg(test)]
mod tests;
mod texture_binding;

use self::cache_identity::PreparedMaterialCacheSlot;
use self::material_readiness::{
    fallback_material_uri, is_standard_texture_slot, material_prepare_result,
    material_uses_renderer_material_abi_fallback, missing_material_fallback_usage,
};

impl ResourceStreamer {
    pub(crate) fn ensure_material(
        &mut self,
        backend: &RenderBackend,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
    ) -> Result<(), GraphicsError> {
        self.ensure_material_internal(backend, device, texture_layout, handle, None)
    }

    pub(crate) fn ensure_material_for_frame(
        &mut self,
        backend: &RenderBackend,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        self.ensure_material_internal(
            backend,
            device,
            texture_layout,
            handle,
            Some(submission_transaction),
        )
    }

    fn ensure_material_internal(
        &mut self,
        backend: &RenderBackend,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
        mut submission_transaction: Option<&mut RenderFrameSubmissionTransaction>,
    ) -> Result<(), GraphicsError> {
        crate::profile_scope!("render", "material", "prepare");
        let id = handle.id();
        let asset_manager = self.asset_manager()?;
        let requested_revision = self.resource_revision(id).ok();
        let texture_support = texture_upload_support_from_device(device);
        let current_slot = self.materials.get(&id).and_then(|prepared| {
            if prepared.published.as_ref().is_some_and(|published| {
                self.prepared_material_bundle_cache_is_current(
                    published,
                    requested_revision,
                    texture_support,
                )
            }) {
                Some(PreparedMaterialCacheSlot::Published)
            } else if prepared.staged_candidate.as_ref().is_some_and(|candidate| {
                self.prepared_material_bundle_cache_is_current(
                    candidate,
                    requested_revision,
                    texture_support,
                )
            }) {
                Some(if prepared.staged_pipeline_failed {
                    PreparedMaterialCacheSlot::RejectedStaged
                } else {
                    PreparedMaterialCacheSlot::Staged
                })
            } else if prepared
                .rejected_candidate
                .as_ref()
                .and_then(|candidate| candidate.identity.as_ref())
                .is_some_and(|identity| {
                    self.prepared_material_candidate_cache_is_current(
                        identity,
                        requested_revision,
                        texture_support,
                    )
                })
            {
                Some(PreparedMaterialCacheSlot::RejectedCandidate)
            } else {
                None
            }
        });
        if let Some(current_slot) = current_slot {
            crate::profile_counter!("render", "material_prepare_cache_hit", 1);
            crate::profile_counter!("render", "material_prepare_rebuild", 0);
            if matches!(
                current_slot,
                PreparedMaterialCacheSlot::Staged
                    | PreparedMaterialCacheSlot::RejectedStaged
                    | PreparedMaterialCacheSlot::RejectedCandidate
            ) {
                crate::profile_counter!("render", "material_candidate_cache_hit", 1);
                crate::profile_counter!(
                    "render",
                    "material_candidate_terminal_cache_hit",
                    if matches!(
                        current_slot,
                        PreparedMaterialCacheSlot::RejectedStaged
                            | PreparedMaterialCacheSlot::RejectedCandidate
                    ) {
                        1
                    } else {
                        0
                    }
                );
                let reactivated = current_slot == PreparedMaterialCacheSlot::Staged
                    && self.active_staged_material_ids.insert(id);
                crate::profile_counter!(
                    "render",
                    "material_candidate_reactivated",
                    if reactivated { 1 } else { 0 }
                );
                return Ok(());
            }
            crate::profile_counter!("render", "material_candidate_cache_hit", 0);
            crate::profile_counter!("render", "material_candidate_terminal_cache_hit", 0);
            let prepared = self
                .materials
                .get_mut(&id)
                .expect("a current prepared material must remain published");
            prepared.rejected_candidate = None;
            prepared.staged_candidate = None;
            prepared.staged_pipeline_failed = false;
            prepared.staged_pipeline_admission_cycle = Default::default();
            let result = material_prepare_result(
                id,
                &prepared
                    .published
                    .as_ref()
                    .expect("the current published cache slot must own a bundle")
                    .runtime
                    .readiness_report,
            );
            self.active_staged_material_ids.remove(&id);
            return result;
        }
        crate::profile_counter!("render", "material_prepare_cache_hit", 0);
        crate::profile_counter!("render", "material_prepare_rebuild", 1);
        crate::profile_counter!("render", "material_candidate_terminal_cache_hit", 0);
        let (
            material,
            missing_material_fallback,
            prepared_revision,
            loaded_material_id,
            loaded_material_revision,
        ) = match asset_manager.load_material_asset_snapshot(id) {
            Ok(material) => {
                let revision = material.revision();
                ((*material).clone(), None, Some(revision), id, revision)
            }
            Err(error) => {
                let fallback_uri = fallback_material_uri();
                let fallback_id = asset_manager.resolve_asset_id(&fallback_uri).ok_or_else(
                        || {
                            GraphicsError::Asset(format!(
                                "missing material {id} ({error}); fallback material {fallback_uri} is not registered"
                            ))
                        },
                    )?;
                let material = asset_manager
                        .load_material_asset_snapshot(fallback_id)
                        .map_err(|fallback_error| {
                            GraphicsError::Asset(format!(
                                "missing material {id} ({error}); fallback material {fallback_uri} failed to load: {fallback_error}"
                            ))
                        })?;
                let fallback_revision = material.revision();
                (
                    (*material).clone(),
                    Some(missing_material_fallback_usage(id)),
                    None,
                    fallback_id,
                    fallback_revision,
                )
            }
        };
        let material_dependency = self
            .prepared_material_dependency_snapshot(loaded_material_id, loaded_material_revision)?;
        let (material, parent_validation_errors) =
            asset_manager.resolve_effective_material_asset(loaded_material_id, material);
        let shader_contract =
            Self::load_shader_contract(asset_manager.as_ref(), material.shader.clone());
        let descriptor = shader_contract
            .as_ref()
            .map(|shader| material.standard_material_descriptor_for_shader(shader.asset()))
            .unwrap_or_else(|| material.standard_material_descriptor());
        let shader_dependency = shader_contract
            .as_ref()
            .map(|shader| PreparedMaterialShaderDependency {
                locator: descriptor.dependencies.shader.locator.clone(),
                id: Some(shader.resource_id()),
                revision: Some(shader.revision()),
                dependency_revision: Some(
                    asset_manager
                        .resource_manager()
                        .readiness_generation()
                        .dependency_revision(shader.resource_id())
                        .unwrap_or(0),
                ),
            })
            .unwrap_or_else(|| {
                self.material_shader_dependency_snapshot(&descriptor.dependencies.shader.locator)
            });
        let texture_dependencies = self.material_texture_dependency_snapshots(
            descriptor.dependencies.textures.iter(),
            texture_support,
        );
        let material_option_bits = shader_contract
            .as_ref()
            .map(|shader| material.material_option_bits_for_shader(shader.asset()))
            .unwrap_or(0);
        let material_layout_hash = shader_contract
            .as_ref()
            .map(|shader| shader.asset().material_property_layout.layout_hash)
            .unwrap_or(0);
        let disabled_passes = shader_contract
            .as_ref()
            .map(|shader| {
                MaterialDisabledPasses::from_shader_pass_names(&shader.asset().disabled_passes)
            })
            .unwrap_or_default();
        let shader_resolver = Arc::clone(&asset_manager);
        let texture_resolver = Arc::clone(&asset_manager);
        let mut readiness = if let Some(shader) = shader_contract.as_ref() {
            material.readiness_report_with_shader_contract(
                shader.asset(),
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
            let shader_resolver = Arc::clone(&asset_manager);
            let texture_resolver = Arc::clone(&asset_manager);
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
        if let Some(shader) = shader_contract.as_ref() {
            if let Some(token) = shader.asset().shading_model.as_deref() {
                if token.parse::<RenderMaterialLightingModel>().is_err() {
                    readiness.push_validation_error_once(
                        RenderMaterialValidationError::UnregisteredShadingModel {
                            path: "shading_model".to_string(),
                            token: token.to_string(),
                        },
                    );
                    readiness.push_fallback_usage_once(RenderMaterialFallbackUsage {
                        reason: RenderMaterialFallbackReason::Validation,
                        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
                    });
                }
            }
        }
        for error in parent_validation_errors {
            readiness.push_validation_error_once(error);
        }
        let uses_renderer_material_abi_fallback = if let Some(shader) = shader_contract.as_ref() {
            let abi_diagnostics = renderer_material_layout_diagnostics(shader.asset());
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
        let shading_model_descriptor = self
            .shading_model_registry
            .resolve_lighting_model(&lighting_model);
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
        let clearcoat_normal_texture = self.resolve_texture_reference_with_support(
            "clearcoat_normal_texture",
            descriptor
                .advanced_features
                .clearcoat_normal_texture
                .as_ref(),
            texture_support,
        );
        let standard_texture_slots = [
            descriptor.base_color_texture.as_ref().map(|_| {
                (
                    "base_color",
                    base_color_texture.id(),
                    Some(base_color_texture.expected_dimension),
                    base_color_texture.actual_dimension,
                    base_color_texture.slot_fallback.clone(),
                )
            }),
            descriptor.normal_texture.as_ref().map(|_| {
                (
                    "normal",
                    normal_texture.id(),
                    Some(normal_texture.expected_dimension),
                    normal_texture.actual_dimension,
                    normal_texture.slot_fallback.clone(),
                )
            }),
            descriptor.metallic_roughness_texture.as_ref().map(|_| {
                (
                    "metallic_roughness",
                    metallic_roughness_texture.id(),
                    Some(metallic_roughness_texture.expected_dimension),
                    metallic_roughness_texture.actual_dimension,
                    metallic_roughness_texture.slot_fallback.clone(),
                )
            }),
            descriptor.occlusion_texture.as_ref().map(|_| {
                (
                    "occlusion",
                    occlusion_texture.id(),
                    Some(occlusion_texture.expected_dimension),
                    occlusion_texture.actual_dimension,
                    occlusion_texture.slot_fallback.clone(),
                )
            }),
            descriptor.emissive_texture.as_ref().map(|_| {
                (
                    "emissive",
                    emissive_texture.id(),
                    Some(emissive_texture.expected_dimension),
                    emissive_texture.actual_dimension,
                    emissive_texture.slot_fallback.clone(),
                )
            }),
            descriptor
                .advanced_features
                .clearcoat_normal_texture
                .as_ref()
                .map(|_| {
                    (
                        "clearcoat_normal",
                        clearcoat_normal_texture.id(),
                        Some(clearcoat_normal_texture.expected_dimension),
                        clearcoat_normal_texture.actual_dimension,
                        clearcoat_normal_texture.slot_fallback.clone(),
                    )
                }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let standard_texture_slot_ids = standard_texture_slots
            .iter()
            .map(|(_, texture_id, _, _, _)| *texture_id)
            .collect::<Vec<_>>();
        let shader_slot_textures = material
            .all_texture_slots()
            .into_iter()
            .filter(|(slot, _)| !is_standard_texture_slot(slot))
            .map(|(slot, texture)| {
                let expected_dimension = shader_contract
                    .as_ref()
                    .and_then(|shader| {
                        shader
                            .asset()
                            .texture_slots
                            .iter()
                            .find(|shader_slot| shader_slot.name == slot)
                    })
                    .map(crate::asset::ShaderTextureSlotAsset::expected_dimension)
                    .unwrap_or(RenderMaterialTextureDimension::D2);
                let resolved = self.resolve_texture_reference_with_dimension_support(
                    &slot,
                    Some(texture),
                    expected_dimension,
                    texture_support,
                );
                (slot, resolved)
            })
            .collect::<Vec<_>>();
        let shader_property_values = shader_contract
            .as_ref()
            .map(|shader| material.shader_property_values_for_shader(shader.asset()))
            .unwrap_or_default();
        let shader_property_value_summary =
            RenderMaterialPropertyValueSummary::from_values(&shader_property_values);
        let shader_property_value_states =
            RenderMaterialPropertyValueState::from_values(&shader_property_values);
        let shader_property_uniform_payload = shader_contract
            .as_ref()
            .map(|shader| {
                RenderMaterialPropertyUniformPayload::from_layout_and_values(
                    &shader.asset().material_property_layout,
                    &shader_property_values,
                )
            })
            .unwrap_or_else(|| {
                RenderMaterialPropertyUniformPayload::from_values(&shader_property_values)
            });
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
            &clearcoat_normal_texture,
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
            RenderMaterialTextureSlotState::from_dimensioned_slots(
                standard_texture_slots.iter().map(
                    |(slot, texture_id, expected, actual, fallback)| {
                        (*slot, *texture_id, *expected, *actual, fallback.clone())
                    },
                ),
            );
        readiness.texture_slot_summary = Some(
            RenderMaterialTextureSlotSummary::from_non_standard_slots(&non_standard_texture_slots),
        );
        readiness.non_standard_texture_slot_states =
            RenderMaterialTextureSlotState::from_dimensioned_slots(
                shader_slot_textures.iter().map(|(slot, texture)| {
                    (
                        slot.clone(),
                        texture.id(),
                        Some(texture.expected_dimension),
                        texture.actual_dimension,
                        texture.slot_fallback.clone(),
                    )
                }),
            );
        let (shader_id, shader_revision, shader_dependency_revision, shader_readiness) =
            match self.ensure_shader_source(&descriptor.dependencies.shader) {
                Ok(shader) => shader,
                Err(error) => {
                    if self
                        .retain_last_good_material_after_candidate_failure(
                            id,
                            readiness,
                            "dependencies.shader",
                            &error,
                        )
                        .is_ok()
                    {
                        return Ok(());
                    }
                    return Err(error);
                }
            };
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
        let (pipeline_shader_id, pipeline_shader_revision, pipeline_shader_dependency_revision) =
            if material_uses_renderer_material_abi_fallback(&readiness.validation_errors) {
                let fallback_key = default_pipeline_key();
                (
                    fallback_key.shader_id,
                    fallback_key.shader_revision,
                    fallback_key.shader_dependency_revision,
                )
            } else {
                (shader_id, shader_revision, shader_dependency_revision)
            };
        let runtime = MaterialRuntime {
            base_color: Vec4::from_array(descriptor.base_color),
            emissive: Vec3::from_array(descriptor.emissive),
            metallic: descriptor.metallic,
            roughness: descriptor.roughness,
            occlusion_strength: descriptor.occlusion_strength,
            normal_scale: descriptor.normal_scale,
            double_sided: descriptor.double_sided,
            alpha_blend,
            alpha_cutoff,
            lighting_model: lighting_model.clone(),
            shading_model_id,
            unlit,
            cast_shadows: descriptor.cast_shadows,
            receive_shadows: descriptor.receive_shadows,
            disabled_passes,
            render_queue: descriptor.render_queue,
            render_queue_value: descriptor.render_queue_value,
            material_queue: descriptor.material_queue,
            depth_bias: descriptor.depth_bias,
            taa_reactive_mask_strength: descriptor.taa_reactive_mask_strength,
            separate_translucency: descriptor.separate_translucency
                && matches!(descriptor.alpha_mode, RenderMaterialAlphaMode::Blend),
            subsurface_profile_index: descriptor.subsurface_profile_index,
            advanced_features: descriptor.advanced_features.clone(),
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
            clearcoat_normal_texture: clearcoat_normal_texture.id(),
            clearcoat_normal_texture_transform: descriptor.clearcoat_normal_texture_transform,
            clearcoat_normal_texture_uv_channel: descriptor.clearcoat_normal_texture_uv_channel,
            shader_property_values,
            shader_property_uniform_payload,
            non_standard_texture_slots,
            pipeline_key: PipelineKey {
                shader_id: pipeline_shader_id,
                shader_revision: pipeline_shader_revision,
                shader_dependency_revision: pipeline_shader_dependency_revision,
                material_layout_hash,
                material_option_bits,
                double_sided: descriptor.double_sided,
                reverse_raster_winding: false,
                alpha_blend,
                alpha_mask,
                alpha_cutoff_bits: alpha_cutoff.map(f32::to_bits),
                receive_shadows: descriptor.receive_shadows,
                shading_model_id,
                unlit,
                has_normal_texture: descriptor.normal_texture.is_some(),
                pbr_clearcoat: descriptor.advanced_features.uses_clearcoat(),
                pbr_anisotropy: descriptor.advanced_features.uses_anisotropy(),
                pbr_ior_override: descriptor.advanced_features.uses_dielectric_f0_override(),
                pbr_transmission: descriptor.advanced_features.uses_transmission(),
                volumetric_fog: false,
            },
            readiness_report: readiness,
        };
        let prepare_result = material_prepare_result(id, &runtime.readiness_report);
        if prepare_result.is_err() {
            runtime.readiness_report = match self.retain_last_good_material_candidate(
                id,
                Some(PreparedMaterialCandidateIdentity::new(
                    prepared_revision,
                    material_dependency,
                    &shader_dependency,
                    &texture_dependencies,
                    texture_support,
                )),
                runtime.readiness_report,
            ) {
                Ok(()) => return Ok(()),
                Err(readiness_report) => readiness_report,
            };
            let uniform = std::sync::Arc::new(GpuMaterialUniformResource::from_payload(
                device,
                &runtime.shader_property_uniform_payload,
            ));
            let standard_uniform = std::sync::Arc::new(
                GpuMaterialUniformResource::from_standard_material(device, &runtime),
            );
            let textures = self.prepared_material_texture_set(&runtime);
            crate::profile_counter!("render", "material_uniform_buffer_creations", 2);
            let draw_generation = self.allocate_material_draw_generation();
            self.active_staged_material_ids.remove(&id);
            self.materials.insert(
                id,
                PreparedMaterial {
                    published: Some(PreparedMaterialBundle {
                        draw_generation,
                        revision: prepared_revision,
                        material_dependency,
                        shader_dependency: shader_dependency.clone(),
                        texture_dependencies: texture_dependencies.clone(),
                        texture_support,
                        runtime,
                        textures,
                        uniform,
                        standard_uniform,
                    }),
                    previous_published: None,
                    staged_candidate: None,
                    staged_pipeline_failed: false,
                    staged_pipeline_admission_cycle: Default::default(),
                    rejected_candidate: None,
                },
            );
            return prepare_result;
        }
        let mut ensured_texture_ids = BTreeSet::new();
        for texture_id in [
            base_color_texture.id(),
            normal_texture.id(),
            metallic_roughness_texture.id(),
            occlusion_texture.id(),
            emissive_texture.id(),
            clearcoat_normal_texture.id(),
        ]
        .into_iter()
        .flatten()
        .chain(
            shader_slot_textures
                .iter()
                .filter_map(|(_slot, texture)| texture.id()),
        ) {
            if ensured_texture_ids.insert(texture_id) {
                if let Err(error) = self.ensure_material_texture(
                    backend,
                    device,
                    texture_layout,
                    texture_id,
                    submission_transaction.as_deref_mut(),
                ) {
                    let path = format!("dependencies.textures[{texture_id}]");
                    if self
                        .retain_last_good_material_after_candidate_failure(
                            id,
                            runtime.readiness_report,
                            path,
                            &error,
                        )
                        .is_ok()
                    {
                        return Ok(());
                    }
                    return Err(error);
                }
            }
        }
        let uniform = std::sync::Arc::new(GpuMaterialUniformResource::from_payload(
            device,
            &runtime.shader_property_uniform_payload,
        ));
        let standard_uniform = std::sync::Arc::new(
            GpuMaterialUniformResource::from_standard_material(device, &runtime),
        );
        let textures = self.prepared_material_texture_set(&runtime);
        crate::profile_counter!("render", "material_uniform_buffer_creations", 2);
        let draw_generation = self.allocate_material_draw_generation();
        self.stage_material_candidate(
            id,
            PreparedMaterialBundle {
                draw_generation,
                revision: prepared_revision,
                material_dependency,
                shader_dependency,
                texture_dependencies,
                texture_support,
                runtime,
                textures,
                uniform,
                standard_uniform,
            },
        );
        Ok(())
    }

    fn allocate_material_draw_generation(&mut self) -> u64 {
        let generation = self.next_material_draw_generation;
        self.next_material_draw_generation = generation.wrapping_add(1).max(1);
        generation
    }

    fn prepared_material_dependency_snapshot(
        &self,
        id: ResourceId,
        revision: u64,
    ) -> Result<PreparedMaterialDependency, GraphicsError> {
        let dependency_revision = self
            .asset_manager()?
            .resource_manager()
            .readiness_generation()
            .dependency_revision(id)
            .unwrap_or(0);
        Ok(PreparedMaterialDependency {
            id,
            revision,
            dependency_revision,
        })
    }

    fn material_dependency_identity_for_id(
        &self,
        id: ResourceId,
    ) -> Option<(ResourceId, u64, u64)> {
        let asset_manager = self.asset_manager().ok()?;
        let resource_manager = asset_manager.resource_manager();
        let revision = resource_manager.registry().get(id)?.revision;
        let dependency_revision = resource_manager
            .readiness_generation()
            .dependency_revision(id)
            .unwrap_or(0);
        Some((id, revision, dependency_revision))
    }

    fn material_shader_dependency_snapshot(
        &self,
        locator: &ResourceLocator,
    ) -> PreparedMaterialShaderDependency {
        let identity = self.shader_dependency_identity_for_locator(locator);
        PreparedMaterialShaderDependency {
            locator: locator.clone(),
            id: identity.map(|(id, _, _)| id),
            revision: identity.map(|(_, revision, _)| revision),
            dependency_revision: identity.map(|(_, _, dependency_revision)| dependency_revision),
        }
    }

    fn shader_dependency_identity_for_locator(
        &self,
        locator: &ResourceLocator,
    ) -> Option<(ResourceId, u64, u64)> {
        let asset_manager = self.asset_manager().ok()?;
        let resource_manager = asset_manager.resource_manager();
        let record = resource_manager
            .registry()
            .get_by_locator(locator)
            .cloned()?;
        let dependency_revision = resource_manager
            .readiness_generation()
            .dependency_revision(record.id())
            .unwrap_or(0);
        Some((record.id(), record.revision, dependency_revision))
    }

    fn texture_dependency_revision_for_locator(
        &self,
        locator: &ResourceLocator,
    ) -> Option<(ResourceId, u64)> {
        self.asset_manager()
            .ok()?
            .resource_manager()
            .registry()
            .get_by_locator(locator)
            .map(|record| (record.id(), record.revision))
    }

    fn material_texture_dependency_snapshots<'a>(
        &self,
        references: impl IntoIterator<Item = &'a AssetReference>,
        texture_support: TextureUploadSupport,
    ) -> Vec<PreparedMaterialTextureDependency> {
        let mut dependencies = Vec::new();
        for reference in references {
            let snapshot =
                self.texture_dependency_snapshot_for_locator(&reference.locator, texture_support);
            if dependencies
                .iter()
                .any(|dependency: &PreparedMaterialTextureDependency| {
                    dependency.locator == snapshot.locator
                })
            {
                continue;
            }
            dependencies.push(snapshot);
        }
        dependencies
    }

    fn texture_dependency_snapshot_for_locator(
        &self,
        locator: &ResourceLocator,
        texture_support: TextureUploadSupport,
    ) -> PreparedMaterialTextureDependency {
        let Ok(asset_manager) = self.asset_manager() else {
            return PreparedMaterialTextureDependency {
                locator: locator.clone(),
                id: None,
                revision: None,
                upload_unsupported_reason: Some("ProjectAssetManager is unavailable".to_string()),
            };
        };
        let Some((texture_id, texture_revision)) = asset_manager
            .resource_manager()
            .registry()
            .get_by_locator(locator)
            .map(|record| (record.id(), record.revision))
        else {
            return PreparedMaterialTextureDependency {
                locator: locator.clone(),
                id: None,
                revision: None,
                upload_unsupported_reason: None,
            };
        };
        let (texture_revision, upload_unsupported_reason) =
            match asset_manager.load_texture_asset_snapshot(texture_id) {
                Ok(texture) => (
                    texture.revision(),
                    texture
                        .upload_readiness(texture_support)
                        .unsupported_reason()
                        .map(str::to_string),
                ),
                Err(error) => (texture_revision, Some(error.to_string())),
            };

        PreparedMaterialTextureDependency {
            locator: locator.clone(),
            id: Some(texture_id),
            revision: Some(texture_revision),
            upload_unsupported_reason,
        }
    }
}
