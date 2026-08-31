use crate::graphics::pipeline::{PipelineAdmission, PipelineAdmissionReason};
use crate::graphics::scene::resources::ResourceStreamer;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_oit_mesh_pipeline;
use super::shader_source::mesh_pipeline_shader_source_for_geometry_descriptor;
use super::shader_source_validation_admission::CachedMeshShaderModule;
use super::{MeshPipelineCache, PipelineCreationTarget};

const OIT_PIPELINE_TARGET: PipelineCreationTarget = PipelineCreationTarget::Oit;

impl MeshPipelineCache {
    pub(crate) fn ensure_oit_pipeline_admission_for_base_variant(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        if self.oit_mesh_variant_pipelines.contains_key(&variant_id) {
            self.clear_pipeline_unavailable_state_for_target(OIT_PIPELINE_TARGET, variant_id);
            return PipelineAdmission::Ready(());
        }
        if let Some(reason) =
            self.pipeline_failure_reason_for_target(OIT_PIPELINE_TARGET, variant_id)
        {
            return self.unavailable_pipeline_for_target(OIT_PIPELINE_TARGET, variant_id, reason);
        }
        let Some((kind, pipeline_key, shader_variant_key)) =
            self.pipeline_and_shader_key_for_variant(variant_id)
        else {
            return self.unavailable_pipeline_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::UnknownVariant,
            );
        };
        if kind != MeshPassPipelineKind::Base || !pipeline_key.is_transparent() {
            return self.unavailable_pipeline_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::WrongPass,
            );
        }
        let Some(geometry_source) =
            self.geometry_source_descriptor_for_variant(&shader_variant_key)
        else {
            self.mark_pipeline_failure_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
                "OIT pipeline geometry source descriptor is unavailable",
            );
            return self.unavailable_pipeline_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
            );
        };
        let shader_source = match mesh_pipeline_shader_source_for_geometry_descriptor(
            streamer,
            &pipeline_key,
            &geometry_source,
        ) {
            Ok(source) => source,
            Err(error) => {
                let message = format!("{error:?}");
                self.record_shader_variant_assembly_error(&shader_variant_key, error);
                self.mark_pipeline_failure_for_target(
                    OIT_PIPELINE_TARGET,
                    variant_id,
                    PipelineAdmissionReason::SourceAssemblyFailed,
                    message,
                );
                return self.unavailable_pipeline_for_target(
                    OIT_PIPELINE_TARGET,
                    variant_id,
                    PipelineAdmissionReason::SourceAssemblyFailed,
                );
            }
        };
        let Some(shader_source) = shader_source.into_oit_fragment_store_source() else {
            self.mark_pipeline_failure_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::OitFragmentStoreUnavailable,
                "shader does not expose the OIT fs_oit fragment-store contract",
            );
            return self.unavailable_pipeline_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::OitFragmentStoreUnavailable,
            );
        };
        self.record_observed_shader_source(OIT_PIPELINE_TARGET, &shader_source.source_hash);
        let shader_key = format!(
            "{}@{}#{}#{}",
            pipeline_key.shader_id,
            pipeline_key.shader_revision,
            shader_variant_key.canonical_string(),
            shader_source.source_hash
        );
        let validated_source = if self.shader_modules.contains_key(&shader_key) {
            None
        } else {
            match self.mesh_pipeline_shader_source_with_cache(
                shader_source,
                &shader_variant_key,
                OIT_PIPELINE_TARGET,
                variant_id,
                &pipeline_key,
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
                &shader_variant_key,
                OIT_PIPELINE_TARGET,
                variant_id,
                &pipeline_key,
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
                label: Some("zircon-oit-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.wgsl_source.into()),
            });
            let creation_elapsed = creation_started.elapsed();
            self.shader_modules.insert(
                shader_key.clone(),
                CachedMeshShaderModule::new(module, source.reflection),
            );
            self.take_ready_shader_source_validation(&validation_key)
                .expect("installed OIT module must consume its validation artifact");
            self.record_shader_module_creation(OIT_PIPELINE_TARGET, creation_elapsed);
        }
        if !self.oit_mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let creation_started = std::time::Instant::now();
            let pipeline = create_oit_mesh_pipeline(
                device,
                &self.oit_mesh_pipeline_layout,
                shader,
                &pipeline_key,
                self.runtime_pipeline_cache.cache(),
            );
            let creation_elapsed = creation_started.elapsed();
            self.oit_mesh_variant_pipelines.insert(variant_id, pipeline);
            self.bind_pipeline_shader_module_reference(
                OIT_PIPELINE_TARGET,
                variant_id,
                &shader_key,
            );
            self.record_render_pipeline_creation(OIT_PIPELINE_TARGET, creation_elapsed);
        }
        let pipeline_validation_failed = self.track_pipeline_creation_error_scope(
            &shader_variant_key,
            OIT_PIPELINE_TARGET,
            variant_id,
            shader_key,
            error_scope,
        );
        if pipeline_validation_failed {
            self.drain_pipeline_creation_diagnostics();
            self.mark_pipeline_failure_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
                "OIT pipeline WGPU validation failed",
            );
            return self.unavailable_pipeline_for_target(
                OIT_PIPELINE_TARGET,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
            );
        }
        self.clear_pipeline_unavailable_state_for_target(OIT_PIPELINE_TARGET, variant_id);
        PipelineAdmission::Ready(())
    }

    pub(crate) fn oit_pipeline_for_ready_base_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> &wgpu::RenderPipeline {
        self.oit_mesh_variant_pipelines
            .get(&variant_id)
            .expect("Ready OIT pipeline admission must retain its pipeline")
    }

    pub(crate) fn oit_fragment_store_layout(&self) -> &wgpu::BindGroupLayout {
        &self.oit_fragment_store_layout
    }
}
