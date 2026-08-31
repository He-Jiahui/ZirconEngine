use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::pipeline::{PipelineAdmission, PipelineAdmissionReason};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_gbuffer_mesh_pipeline;
use super::shader_source::mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer;
use super::shader_source_validation_admission::CachedMeshShaderModule;
use super::{MeshPipelineCache, PipelineCreationTarget};

const GBUFFER_MESH_SHADER_KEY_PREFIX: &str = "zircon.builtin.deferred-gbuffer-mesh@1";
const GBUFFER_PIPELINE_TARGET: PipelineCreationTarget =
    PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer);

impl MeshPipelineCache {
    pub(crate) fn gbuffer_variant_admission_for_command_variant(
        &mut self,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<MeshPipelineVariantId> {
        let Some((kind, pipeline_key, shader_variant_key)) =
            self.pipeline_and_shader_key_for_variant(variant_id)
        else {
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::UnknownVariant,
            );
        };
        match kind {
            MeshPassPipelineKind::GBuffer => PipelineAdmission::Ready(variant_id),
            MeshPassPipelineKind::Base => {
                PipelineAdmission::Ready(self.resolve_variant_for_geometry(
                    MeshPassPipelineKind::GBuffer,
                    &pipeline_key,
                    shader_variant_key.geometry_source,
                    shader_variant_key.quality,
                ))
            }
            _ => self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::WrongPass,
            ),
        }
    }

    fn ensure_gbuffer_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        key: &PipelineKey,
        shader_variant_key: &ShaderVariantKey,
    ) -> PipelineAdmission<()> {
        let Some(geometry_source) = self.geometry_source_descriptor_for_variant(shader_variant_key)
        else {
            self.mark_pipeline_failure_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
                "GBuffer pipeline geometry source descriptor is unavailable",
            );
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
            );
        };
        let shader_source =
            match mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                key,
                &geometry_source,
            ) {
                Ok(source) => source,
                Err(error) => {
                    let message = format!("{error:?}");
                    self.record_shader_variant_assembly_error(shader_variant_key, error);
                    self.mark_pipeline_failure_for_target(
                        GBUFFER_PIPELINE_TARGET,
                        variant_id,
                        PipelineAdmissionReason::SourceAssemblyFailed,
                        message,
                    );
                    return self.unavailable_pipeline_for_target(
                        GBUFFER_PIPELINE_TARGET,
                        variant_id,
                        PipelineAdmissionReason::SourceAssemblyFailed,
                    );
                }
        };
        self.record_observed_shader_source(GBUFFER_PIPELINE_TARGET, &shader_source.source_hash);
        let shader_key = gbuffer_mesh_shader_key(shader_variant_key, &shader_source.source_hash);
        let validated_source = if self.shader_modules.contains_key(&shader_key) {
            None
        } else {
            match self.mesh_pipeline_shader_source_with_cache(
                shader_source,
                shader_variant_key,
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                key,
            ) {
                PipelineAdmission::Ready(source) => Some(source),
                PipelineAdmission::Deferred(unavailable) => {
                    return PipelineAdmission::Deferred(unavailable);
                }
                PipelineAdmission::Failed(unavailable) => {
                    return PipelineAdmission::Failed(unavailable);
                }
            }
        };
        if validated_source.is_none() {
            match self.cached_shader_module_entry_admission(
                &shader_key,
                shader_variant_key,
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                key,
            ) {
                PipelineAdmission::Ready(()) => {}
                PipelineAdmission::Deferred(unavailable) => {
                    return PipelineAdmission::Deferred(unavailable);
                }
                PipelineAdmission::Failed(unavailable) => {
                    return PipelineAdmission::Failed(unavailable);
                }
            }
        }
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if let Some(source) = validated_source {
            let validation_key = source.validation_key;
            let creation_started = std::time::Instant::now();
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-deferred-gbuffer-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.wgsl_source.into()),
            });
            let creation_elapsed = creation_started.elapsed();
            self.shader_modules.insert(
                shader_key.clone(),
                CachedMeshShaderModule::new(module, source.reflection),
            );
            self.take_ready_shader_source_validation(&validation_key)
                .expect("installed GBuffer module must consume its validation artifact");
            self.record_shader_module_creation(GBUFFER_PIPELINE_TARGET, creation_elapsed);
        }
        if !self.gbuffer_mesh_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("deferred gbuffer mesh shader module cached");
            let creation_started = std::time::Instant::now();
            let pipeline = create_gbuffer_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                key,
                self.runtime_pipeline_cache.cache(),
            );
            let creation_elapsed = creation_started.elapsed();
            self.gbuffer_mesh_pipelines.insert(variant_id, pipeline);
            self.bind_pipeline_shader_module_reference(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                &shader_key,
            );
            self.record_render_pipeline_creation(GBUFFER_PIPELINE_TARGET, creation_elapsed);
        }
        let pipeline_validation_failed = self.track_pipeline_creation_error_scope(
            shader_variant_key,
            GBUFFER_PIPELINE_TARGET,
            variant_id,
            shader_key,
            error_scope,
        );
        if pipeline_validation_failed {
            self.drain_pipeline_creation_diagnostics();
            self.mark_pipeline_failure_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
                "GBuffer pipeline WGPU validation failed",
            );
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
            );
        }
        self.clear_pipeline_unavailable_state_for_target(GBUFFER_PIPELINE_TARGET, variant_id);
        PipelineAdmission::Ready(())
    }

    pub(crate) fn ensure_gbuffer_pipeline_admission_for_variant(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        if self.gbuffer_mesh_pipelines.contains_key(&variant_id) {
            self.clear_pipeline_unavailable_state_for_target(GBUFFER_PIPELINE_TARGET, variant_id);
            return PipelineAdmission::Ready(());
        }
        if let Some(reason) =
            self.pipeline_failure_reason_for_target(GBUFFER_PIPELINE_TARGET, variant_id)
        {
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                reason,
            );
        }
        let Some((kind, pipeline_key, shader_variant_key)) =
            self.pipeline_and_shader_key_for_variant(variant_id)
        else {
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::UnknownVariant,
            );
        };
        if kind != MeshPassPipelineKind::GBuffer {
            return self.unavailable_pipeline_for_target(
                GBUFFER_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::WrongPass,
            );
        }
        self.ensure_gbuffer_pipeline(
            device,
            streamer,
            variant_id,
            &pipeline_key,
            &shader_variant_key,
        )
    }

    pub(crate) fn gbuffer_pipeline_for_ready_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> &wgpu::RenderPipeline {
        self.gbuffer_mesh_pipelines
            .get(&variant_id)
            .expect("Ready GBuffer pipeline admission must retain its pipeline")
    }
}

fn gbuffer_mesh_shader_key(variant_key: &ShaderVariantKey, source_hash: &str) -> String {
    format!(
        "{}#{}#{}",
        GBUFFER_MESH_SHADER_KEY_PREFIX,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPassType;
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::super::mesh_pipeline_deferred_gbuffer_template_source_for_geometry;
    use super::{GBUFFER_MESH_SHADER_KEY_PREFIX, gbuffer_mesh_shader_key};

    #[test]
    fn gbuffer_mesh_shader_key_includes_shader_variant_identity_and_source_hash() {
        let variant_key =
            default_pipeline_key().shader_variant_key(ShaderPassType::GBuffer, "wgpu-runtime");
        let source = match mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
            &default_pipeline_key(),
            variant_key.geometry_source,
        ) {
            Ok(source) => source,
            Err(error) => panic!("deferred gbuffer template source assembly failed: {error:?}"),
        };
        let key = gbuffer_mesh_shader_key(&variant_key, &source.source_hash);

        assert!(key.starts_with(GBUFFER_MESH_SHADER_KEY_PREFIX));
        assert!(key.contains(&variant_key.canonical_string()));
        assert!(key.contains(&source.source_hash));
    }
}
