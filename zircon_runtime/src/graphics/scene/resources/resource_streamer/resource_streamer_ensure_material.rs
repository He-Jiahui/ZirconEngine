use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::asset::{AssetReference, MaterialAsset, ShaderAsset, TextureUploadSupport};
use crate::core::framework::render::{
    RenderImageUsage, RenderMaterialAlphaMode, RenderMaterialFallbackPolicy,
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialLightingModel,
    RenderMaterialPropertyUniformPayload, RenderMaterialPropertyValueState,
    RenderMaterialPropertyValueSummary, RenderMaterialTextureDimension,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialValidationError, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, ResourceLocator};

use crate::graphics::types::GraphicsError;

use super::super::prepared::{PreparedMaterial, PreparedMaterialTextureDependency};
use super::super::{
    default_pipeline_key, texture_upload_support_from_device, GpuMaterialUniformResource,
    MaterialDisabledPasses, MaterialRuntime, PipelineKey,
};
use super::resource_streamer_validate_material_shader_layout::renderer_material_layout_diagnostics;
use super::ResourceStreamer;

mod material_readiness;
#[cfg(test)]
mod tests;

use self::material_readiness::{
    fallback_material_uri, invalid_parent_diagnostic, is_standard_texture_slot,
    material_prepare_result, material_uses_renderer_material_abi_fallback,
    missing_material_fallback_usage, prepared_material_cache_identity_is_current,
};

const MAX_MATERIAL_PARENT_DEPTH: usize = 4;

impl ResourceStreamer {
    pub(crate) fn ensure_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let asset_manager = self.asset_manager()?;
        let requested_revision = self.resource_revision(id).ok();
        let texture_support = texture_upload_support_from_device(device);
        if let Some(prepared) = self.materials.get(&id).filter(|prepared| {
            self.prepared_material_cache_is_current(prepared, requested_revision, texture_support)
        }) {
            return material_prepare_result(id, &prepared.runtime.readiness_report);
        }
        let (material, missing_material_fallback, prepared_revision, loaded_material_id) =
            match asset_manager.load_material_asset(id) {
                Ok(material) => (material, None, requested_revision, id),
                Err(error) => {
                    let fallback_uri = fallback_material_uri();
                    let fallback_id = asset_manager.resolve_asset_id(&fallback_uri).ok_or_else(
                        || {
                            GraphicsError::Asset(format!(
                                "missing material {id} ({error}); fallback material {fallback_uri} is not registered"
                            ))
                        },
                    )?;
                    let material = asset_manager.load_material_asset(fallback_id).map_err(
                        |fallback_error| {
                            GraphicsError::Asset(format!(
                                "missing material {id} ({error}); fallback material {fallback_uri} failed to load: {fallback_error}"
                            ))
                        },
                    )?;
                    (
                        material,
                        Some(missing_material_fallback_usage(id)),
                        None,
                        fallback_id,
                    )
                }
            };
        let (material, parent_validation_errors) =
            self.material_with_parent_chain(asset_manager.as_ref(), loaded_material_id, material);
        let shader_contract =
            Self::load_shader_contract(asset_manager.as_ref(), material.shader.clone());
        let descriptor = shader_contract
            .as_ref()
            .map(|shader| material.standard_material_descriptor_for_shader(shader))
            .unwrap_or_else(|| material.standard_material_descriptor());
        let texture_dependencies = self.material_texture_dependency_snapshots(
            descriptor.dependencies.textures.iter(),
            texture_support,
        );
        let material_option_bits = shader_contract
            .as_ref()
            .map(|shader| material.material_option_bits_for_shader(shader))
            .unwrap_or(0);
        let material_layout_hash = shader_contract
            .as_ref()
            .map(|shader| shader.material_property_layout.layout_hash)
            .unwrap_or(0);
        let disabled_passes = shader_contract
            .as_ref()
            .map(|shader| MaterialDisabledPasses::from_shader_pass_names(&shader.disabled_passes))
            .unwrap_or_default();
        let shader_resolver = Arc::clone(&asset_manager);
        let texture_resolver = Arc::clone(&asset_manager);
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
            if let Some(token) = shader.shading_model.as_deref() {
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
            .map(|shader| material.shader_property_values_for_shader(shader))
            .unwrap_or_default();
        let shader_property_value_summary =
            RenderMaterialPropertyValueSummary::from_values(&shader_property_values);
        let shader_property_value_states =
            RenderMaterialPropertyValueState::from_values(&shader_property_values);
        let shader_property_uniform_payload = shader_contract
            .as_ref()
            .map(|shader| {
                RenderMaterialPropertyUniformPayload::from_layout_and_values(
                    &shader.material_property_layout,
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
            shader_property_values,
            shader_property_uniform_payload,
            non_standard_texture_slots,
            pipeline_key: PipelineKey {
                shader_id: pipeline_shader_id,
                shader_revision: pipeline_shader_revision,
                material_layout_hash,
                material_option_bits,
                double_sided: descriptor.double_sided,
                alpha_blend,
                alpha_mask,
                alpha_cutoff_bits: alpha_cutoff.map(f32::to_bits),
                receive_shadows: descriptor.receive_shadows,
                shading_model_id,
                unlit,
                has_base_color_texture: descriptor.base_color_texture.is_some(),
                has_normal_texture: descriptor.normal_texture.is_some(),
                has_metallic_roughness_texture: descriptor.metallic_roughness_texture.is_some(),
                has_occlusion_texture: descriptor.occlusion_texture.is_some(),
                has_emissive_texture: descriptor.emissive_texture.is_some(),
                pbr_clearcoat: descriptor.advanced_features.uses_clearcoat(),
                pbr_anisotropy: descriptor.advanced_features.uses_anisotropy(),
                pbr_transmission: descriptor.advanced_features.uses_transmission(),
                volumetric_fog: false,
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
                    texture_dependencies: texture_dependencies.clone(),
                    texture_support,
                    runtime,
                    uniform,
                    standard_uniform,
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
                self.ensure_material_texture(device, queue, texture_layout, texture_id)?;
            }
        }
        self.materials.insert(
            id,
            PreparedMaterial {
                revision: prepared_revision,
                texture_dependencies,
                texture_support,
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

    fn prepared_material_cache_is_current(
        &self,
        prepared: &PreparedMaterial,
        requested_revision: Option<u64>,
        texture_support: TextureUploadSupport,
    ) -> bool {
        prepared_material_cache_identity_is_current(
            prepared.revision,
            requested_revision,
            prepared.texture_support,
            texture_support,
            &prepared.texture_dependencies,
            |locator| self.texture_dependency_revision_for_locator(locator),
        )
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
        let upload_unsupported_reason = match asset_manager.load_texture_asset(texture_id) {
            Ok(texture) => texture
                .upload_readiness(texture_support)
                .unsupported_reason()
                .map(str::to_string),
            Err(error) => Some(error.to_string()),
        };

        PreparedMaterialTextureDependency {
            locator: locator.clone(),
            id: Some(texture_id),
            revision: Some(texture_revision),
            upload_unsupported_reason,
        }
    }

    fn load_shader_contract(
        asset_manager: &crate::asset::ProjectAssetManager,
        reference: AssetReference,
    ) -> Option<ShaderAsset> {
        asset_manager
            .resolve_asset_id(&reference.locator)
            .and_then(|id| asset_manager.load_shader_asset(id).ok())
    }

    fn material_with_parent_chain(
        &self,
        asset_manager: &crate::asset::ProjectAssetManager,
        root_id: ResourceId,
        material: MaterialAsset,
    ) -> (MaterialAsset, Vec<RenderMaterialValidationError>) {
        let root_shader = material.shader.clone();
        let mut diagnostics = Vec::new();
        let mut visited = BTreeSet::from([root_id]);
        let mut lineage = vec![(root_id, material)];

        loop {
            let Some(parent_reference) = lineage
                .last()
                .and_then(|(_, material)| material.parent.clone())
            else {
                break;
            };
            if lineage.len() > MAX_MATERIAL_PARENT_DEPTH {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain exceeds depth limit {MAX_MATERIAL_PARENT_DEPTH}"
                )));
                break;
            }
            let Some(parent_id) = asset_manager.resolve_asset_id(&parent_reference.locator) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` is not registered",
                    parent_reference.locator
                )));
                break;
            };
            if !visited.insert(parent_id) {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain contains cycle at {parent_id}"
                )));
                break;
            }
            let Ok(parent) = asset_manager.load_material_asset(parent_id) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` failed to load",
                    parent_reference.locator
                )));
                break;
            };
            if parent.shader != root_shader {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` uses shader `{}` but child uses `{}`",
                    parent_reference.locator, parent.shader.locator, root_shader.locator
                )));
                break;
            }
            lineage.push((parent_id, parent));
        }

        let mut effective = lineage
            .pop()
            .map(|(_, material)| material)
            .expect("material lineage contains root");
        while let Some((_, mut child)) = lineage.pop() {
            child.inherit_parent_values_from(&effective);
            effective = child;
        }
        effective.parent = None;
        (effective, diagnostics)
    }
}
