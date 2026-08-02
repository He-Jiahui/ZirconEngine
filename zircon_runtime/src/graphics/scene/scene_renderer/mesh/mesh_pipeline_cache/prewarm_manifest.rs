use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest,
};
use crate::graphics::scene::resources::{PipelineKey, default_pipeline_key};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{
    create_depth_prepass_mesh_pipeline, create_gbuffer_mesh_pipeline, create_mesh_pipeline,
    create_oit_mesh_pipeline, create_shadow_mesh_pipeline, create_taa_reactive_mask_mesh_pipeline,
    create_taa_reactive_material_mask_mesh_pipeline, create_velocity_mesh_pipeline,
};
use super::{MeshPipelineCache, MeshPipelineShaderSource};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeShaderPipelinePrewarmReport {
    requested_count: usize,
    ready_count: usize,
    cache_hit_count: usize,
    failed_count: usize,
    failures: Vec<RuntimeShaderPipelinePrewarmFailure>,
}

impl RuntimeShaderPipelinePrewarmReport {
    pub const fn requested_count(&self) -> usize {
        self.requested_count
    }

    pub const fn ready_count(&self) -> usize {
        self.ready_count
    }

    pub const fn cache_hit_count(&self) -> usize {
        self.cache_hit_count
    }

    pub const fn failed_count(&self) -> usize {
        self.failed_count
    }

    pub fn failures(&self) -> &[RuntimeShaderPipelinePrewarmFailure] {
        &self.failures
    }

    fn record_ready(&mut self, cache_hit: bool) {
        self.ready_count += 1;
        if cache_hit {
            self.cache_hit_count += 1;
        }
    }

    fn record_failure(&mut self, variant_index: usize, error: impl Into<String>) {
        self.failed_count += 1;
        self.failures.push(RuntimeShaderPipelinePrewarmFailure {
            variant_index,
            error: error.into(),
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeShaderPipelinePrewarmFailure {
    variant_index: usize,
    error: String,
}

impl RuntimeShaderPipelinePrewarmFailure {
    pub const fn variant_index(&self) -> usize {
        self.variant_index
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

impl MeshPipelineCache {
    pub(crate) fn prewarm_manifest(
        &mut self,
        device: &wgpu::Device,
        manifest: &ShaderVariantPrewarmManifest,
    ) -> RuntimeShaderPipelinePrewarmReport {
        let mut report = RuntimeShaderPipelinePrewarmReport {
            requested_count: manifest.variants.len(),
            ..Default::default()
        };
        if manifest.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
            for variant_index in 0..manifest.variants.len() {
                report.record_failure(
                    variant_index,
                    format!(
                        "shader pipeline prewarm manifest schema {} is not supported; expected {}",
                        manifest.schema_version,
                        ShaderVariantPrewarmManifest::SCHEMA_VERSION
                    ),
                );
            }
            return report;
        }

        for (variant_index, request) in manifest.variants.iter().enumerate() {
            if self
                .geometry_source_descriptor(request.key.geometry_source)
                .is_none()
            {
                report.record_failure(
                    variant_index,
                    format!(
                        "geometry source {} is not registered by the runtime renderer",
                        request.key.geometry_source.value()
                    ),
                );
                continue;
            }
            let pipeline_key = match pipeline_key_from_prewarm_request(request) {
                Ok(pipeline_key) => pipeline_key,
                Err(error) => {
                    report.record_failure(variant_index, error);
                    continue;
                }
            };
            let pipeline_kind = pipeline_kind_from_prewarm_request(request);
            let variant_id = self.resolve_variant_for_geometry(
                pipeline_kind,
                &pipeline_key,
                request.key.geometry_source,
                request.key.quality,
            );
            let companion = (pipeline_kind == MeshPassPipelineKind::TaaReactiveMask).then(|| {
                let kind = MeshPassPipelineKind::TaaReactiveMaterialMask;
                let variant_id = self.resolve_variant_for_geometry(
                    kind,
                    &pipeline_key,
                    request.key.geometry_source,
                    request.key.quality,
                );
                (kind, variant_id)
            });
            let primary_hit = self.has_pipeline(pipeline_kind, variant_id);
            let companion_hit =
                companion.is_none_or(|(kind, variant_id)| self.has_pipeline(kind, variant_id));
            let requires_oit =
                pipeline_kind == MeshPassPipelineKind::Base && pipeline_key.is_transparent();
            let oit_hit =
                !requires_oit || self.oit_mesh_variant_pipelines.contains_key(&variant_id);
            if primary_hit && companion_hit && oit_hit {
                report.record_ready(true);
                continue;
            }

            let oit_source = if requires_oit && !oit_hit {
                let source = MeshPipelineShaderSource {
                    wgsl_source: request.wgsl_source.clone(),
                    source_hash: String::new(),
                    cache_content_hashes: request.include_content_hashes.clone(),
                    template_revision: request.template_revision.clone(),
                    segments: Vec::new(),
                };
                match source.into_oit_fragment_store_source() {
                    Some(source) => Some(source),
                    None => {
                        report.record_failure(
                            variant_index,
                            "transparent Forward prewarm source cannot produce the OIT fragment-store entry",
                        );
                        continue;
                    }
                }
            } else {
                None
            };

            let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
            let shader = (!primary_hit || !companion_hit).then(|| {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("zircon-runtime-shader-pipeline-prewarm-module"),
                    source: wgpu::ShaderSource::Wgsl(request.wgsl_source.as_str().into()),
                })
            });
            let (primary_pipeline, companion_pipeline) = match shader.as_ref() {
                Some(shader) => {
                    let primary_pipeline = (!primary_hit).then(|| {
                        self.create_prewarmed_pipeline(device, shader, pipeline_kind, &pipeline_key)
                    });
                    let companion_pipeline = companion.and_then(|(kind, variant_id)| {
                        (!companion_hit).then(|| {
                            (
                                kind,
                                variant_id,
                                self.create_prewarmed_pipeline(device, shader, kind, &pipeline_key),
                            )
                        })
                    });
                    (primary_pipeline, companion_pipeline)
                }
                None if primary_hit && companion_hit => (None, None),
                None => {
                    report.record_failure(
                        variant_index,
                        "missing pipeline prewarm shader module for an uncached standard pass",
                    );
                    continue;
                }
            };
            let oit_pipeline = oit_source.map(|source| {
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("zircon-runtime-oit-shader-pipeline-prewarm-module"),
                    source: wgpu::ShaderSource::Wgsl(source.wgsl_source.as_str().into()),
                });
                create_oit_mesh_pipeline(
                    device,
                    &self.oit_mesh_pipeline_layout,
                    &shader,
                    &pipeline_key,
                    self.runtime_pipeline_cache.cache(),
                )
            });
            if let Some(error) = pollster::block_on(error_scope.pop()) {
                report.record_failure(variant_index, error.to_string());
                continue;
            }
            if let Some(pipeline) = primary_pipeline {
                self.insert_prewarmed_pipeline(pipeline_kind, variant_id, pipeline);
            }
            if let Some((kind, variant_id, pipeline)) = companion_pipeline {
                self.insert_prewarmed_pipeline(kind, variant_id, pipeline);
            }
            if let Some(pipeline) = oit_pipeline {
                self.oit_mesh_variant_pipelines.insert(variant_id, pipeline);
            }
            report.record_ready(false);
        }
        report
    }

    fn has_pipeline(
        &self,
        pipeline_kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
    ) -> bool {
        match pipeline_kind {
            MeshPassPipelineKind::Base => self.mesh_variant_pipelines.contains_key(&variant_id),
            MeshPassPipelineKind::GBuffer => self.gbuffer_mesh_pipelines.contains_key(&variant_id),
            MeshPassPipelineKind::DepthPrepass => {
                self.depth_prepass_mesh_pipelines.contains_key(&variant_id)
            }
            MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => {
                self.shadow_mesh_pipelines.contains_key(&variant_id)
            }
            MeshPassPipelineKind::Velocity => {
                self.velocity_mesh_pipelines.contains_key(&variant_id)
            }
            MeshPassPipelineKind::TaaReactiveMask => self
                .taa_reactive_mask_mesh_pipelines
                .contains_key(&variant_id),
            MeshPassPipelineKind::TaaReactiveMaterialMask => self
                .taa_reactive_material_mask_mesh_pipelines
                .contains_key(&variant_id),
        }
    }

    fn create_prewarmed_pipeline(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> wgpu::RenderPipeline {
        let pipeline_cache = self.runtime_pipeline_cache.cache();
        match pipeline_kind {
            MeshPassPipelineKind::Base => create_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                self.target_format,
                pipeline_key,
                pipeline_cache,
            ),
            MeshPassPipelineKind::GBuffer => create_gbuffer_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                pipeline_key,
                pipeline_cache,
            ),
            MeshPassPipelineKind::DepthPrepass => create_depth_prepass_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                pipeline_key,
                pipeline_cache,
            ),
            MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => {
                create_shadow_mesh_pipeline(
                    device,
                    &self.mesh_pipeline_layout,
                    shader,
                    pipeline_kind,
                    pipeline_cache,
                )
            }
            MeshPassPipelineKind::Velocity => create_velocity_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::Rg16Float,
                pipeline_key,
                pipeline_cache,
            ),
            MeshPassPipelineKind::TaaReactiveMask => create_taa_reactive_mask_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::R8Unorm,
                pipeline_key,
                pipeline_cache,
            ),
            MeshPassPipelineKind::TaaReactiveMaterialMask => {
                create_taa_reactive_material_mask_mesh_pipeline(
                    device,
                    &self.mesh_pipeline_layout,
                    shader,
                    wgpu::TextureFormat::R8Unorm,
                    pipeline_key,
                    pipeline_cache,
                )
            }
        }
    }

    fn insert_prewarmed_pipeline(
        &mut self,
        pipeline_kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
        pipeline: wgpu::RenderPipeline,
    ) {
        match pipeline_kind {
            MeshPassPipelineKind::Base => {
                self.mesh_variant_pipelines.insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::GBuffer => {
                self.gbuffer_mesh_pipelines.insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::DepthPrepass => {
                self.depth_prepass_mesh_pipelines
                    .insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => {
                self.shadow_mesh_pipelines.insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::Velocity => {
                self.velocity_mesh_pipelines.insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::TaaReactiveMask => {
                self.taa_reactive_mask_mesh_pipelines
                    .insert(variant_id, pipeline);
            }
            MeshPassPipelineKind::TaaReactiveMaterialMask => {
                self.taa_reactive_material_mask_mesh_pipelines
                    .insert(variant_id, pipeline);
            }
        }
    }
}

pub(super) fn pipeline_key_from_prewarm_request(
    request: &ShaderVariantPrewarmRequest,
) -> Result<PipelineKey, &'static str> {
    let pipeline_state = request
        .pipeline_state
        .ok_or("runtime shader pipeline prewarm requires the exact pipeline_state descriptor")?;
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.shader_id = request.key.material_shader;
    pipeline_key.shader_revision = request.key.material_revision;
    pipeline_key.material_layout_hash = request.key.material_layout_hash;
    pipeline_key.material_option_bits = request.key.material_option_bits;
    pipeline_key.double_sided = request
        .key
        .features
        .contains(ShaderFeatureBits::DOUBLE_SIDED);
    pipeline_key.alpha_mask = request.key.features.contains(ShaderFeatureBits::ALPHA_TEST);
    pipeline_key.alpha_blend = pipeline_state.alpha_blend;
    pipeline_key.alpha_cutoff_bits = pipeline_state.alpha_cutoff_bits;
    pipeline_key.receive_shadows = request
        .key
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS);
    pipeline_key.has_normal_texture = request
        .key
        .features
        .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE);
    pipeline_key.unlit = pipeline_state.unlit;
    pipeline_key.has_base_color_texture = pipeline_state.has_base_color_texture;
    pipeline_key.has_metallic_roughness_texture = pipeline_state.has_metallic_roughness_texture;
    pipeline_key.has_occlusion_texture = pipeline_state.has_occlusion_texture;
    pipeline_key.has_emissive_texture = pipeline_state.has_emissive_texture;
    pipeline_key.pbr_clearcoat = request
        .key
        .features
        .contains(ShaderFeatureBits::PBR_CLEARCOAT);
    pipeline_key.pbr_anisotropy = request
        .key
        .features
        .contains(ShaderFeatureBits::PBR_ANISOTROPY);
    pipeline_key.pbr_transmission = request
        .key
        .features
        .contains(ShaderFeatureBits::PBR_TRANSMISSION);
    pipeline_key.volumetric_fog = request
        .key
        .features
        .contains(ShaderFeatureBits::VOLUMETRIC_FOG);
    pipeline_key.shading_model_id = request.key.shading_model;
    Ok(pipeline_key)
}

pub(super) fn pipeline_kind_from_prewarm_request(
    request: &ShaderVariantPrewarmRequest,
) -> MeshPassPipelineKind {
    match request.key.pass_type {
        ShaderPassType::Forward => MeshPassPipelineKind::Base,
        ShaderPassType::GBuffer => MeshPassPipelineKind::GBuffer,
        ShaderPassType::DepthPrepass => MeshPassPipelineKind::DepthPrepass,
        ShaderPassType::Shadow if request.key.features.contains(ShaderFeatureBits::ALPHA_TEST) => {
            MeshPassPipelineKind::ShadowDepthAlphaMask
        }
        ShaderPassType::Shadow => MeshPassPipelineKind::ShadowDepth,
        ShaderPassType::Velocity => MeshPassPipelineKind::Velocity,
        ShaderPassType::TaaReactiveMask => MeshPassPipelineKind::TaaReactiveMask,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        ShaderPassType, ShaderPipelinePrewarmState, ShaderVariantPrewarmRequest,
    };
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::pipeline_key_from_prewarm_request;

    #[test]
    fn runtime_prewarm_pipeline_state_reconstructs_complete_pipeline_key() {
        let mut expected = default_pipeline_key();
        expected.shader_revision = 19;
        expected.material_layout_hash = 23;
        expected.material_option_bits = 29;
        expected.double_sided = true;
        expected.alpha_blend = true;
        expected.alpha_mask = true;
        expected.alpha_cutoff_bits = Some(0.42_f32.to_bits());
        expected.receive_shadows = false;
        expected.unlit = true;
        expected.has_base_color_texture = true;
        expected.has_normal_texture = true;
        expected.has_metallic_roughness_texture = true;
        expected.has_occlusion_texture = true;
        expected.has_emissive_texture = true;
        expected.pbr_clearcoat = true;
        expected.pbr_anisotropy = true;
        expected.pbr_transmission = true;
        expected.volumetric_fog = true;
        let request = ShaderVariantPrewarmRequest {
            key: expected.shader_variant_key(ShaderPassType::Forward, "wgpu-runtime"),
            pipeline_state: Some(ShaderPipelinePrewarmState {
                alpha_blend: expected.alpha_blend,
                alpha_cutoff_bits: expected.alpha_cutoff_bits,
                unlit: expected.unlit,
                has_base_color_texture: expected.has_base_color_texture,
                has_metallic_roughness_texture: expected.has_metallic_roughness_texture,
                has_occlusion_texture: expected.has_occlusion_texture,
                has_emissive_texture: expected.has_emissive_texture,
            }),
            source_label: String::new(),
            wgsl_source: String::new(),
            include_content_hashes: Vec::new(),
            template_revision: String::new(),
            naga_version: String::new(),
            wgpu_version: String::new(),
        };

        assert_eq!(pipeline_key_from_prewarm_request(&request), Ok(expected));
    }

    #[test]
    fn runtime_prewarm_rejects_manifest_request_without_exact_pipeline_state() {
        let key = default_pipeline_key();
        let request = ShaderVariantPrewarmRequest {
            key: key.shader_variant_key(ShaderPassType::Forward, "wgpu-runtime"),
            pipeline_state: None,
            source_label: String::new(),
            wgsl_source: String::new(),
            include_content_hashes: Vec::new(),
            template_revision: String::new(),
            naga_version: String::new(),
            wgpu_version: String::new(),
        };

        assert_eq!(
            pipeline_key_from_prewarm_request(&request),
            Err("runtime shader pipeline prewarm requires the exact pipeline_state descriptor")
        );
    }
}
