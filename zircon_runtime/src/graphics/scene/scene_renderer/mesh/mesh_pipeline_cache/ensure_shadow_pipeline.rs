use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::pipeline::{PipelineAdmission, PipelineAdmissionReason};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_shadow_mesh_pipeline;
use super::shader_source::mesh_pipeline_shadow_template_source_for_geometry_descriptor_with_streamer;
use super::shader_source_validation_admission::CachedMeshShaderModule;
use super::{MeshPipelineCache, PipelineCreationTarget};

const SHADOW_MESH_SHADER_KEY_PREFIX: &str = "zircon.builtin.shadow-mesh@1";

impl MeshPipelineCache {
    fn ensure_shadow_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        key: &PipelineKey,
        kind: MeshPassPipelineKind,
        shader_variant_key: &ShaderVariantKey,
    ) -> PipelineAdmission<()> {
        let target = PipelineCreationTarget::MeshPass(kind);
        let Some(geometry_source) = self.geometry_source_descriptor_for_variant(shader_variant_key)
        else {
            self.mark_pipeline_failure_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
                "Shadow pipeline geometry source descriptor is unavailable",
            );
            return self.unavailable_pipeline_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
            );
        };
        let shader_source =
            match mesh_pipeline_shadow_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                key,
                &geometry_source,
            ) {
                Ok(source) => source,
                Err(error) => {
                    let message = format!("{error:?}");
                    self.record_shader_variant_assembly_error(shader_variant_key, error);
                    self.mark_pipeline_failure_for_target(
                        target,
                        variant_id,
                        PipelineAdmissionReason::SourceAssemblyFailed,
                        message,
                    );
                    return self.unavailable_pipeline_for_target(
                        target,
                        variant_id,
                        PipelineAdmissionReason::SourceAssemblyFailed,
                    );
                }
            };
        self.record_observed_shader_source(target, &shader_source.source_hash);
        let shader_key = shadow_mesh_shader_key(shader_variant_key, &shader_source.source_hash);
        let validated_source = if self.shader_modules.contains_key(&shader_key) {
            None
        } else {
            match self.mesh_pipeline_shader_source_with_cache(
                shader_source,
                shader_variant_key,
                target,
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
                target,
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
                label: Some("zircon-shadow-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.wgsl_source.into()),
            });
            let creation_elapsed = creation_started.elapsed();
            self.shader_modules.insert(
                shader_key.clone(),
                CachedMeshShaderModule::new(module, source.reflection),
            );
            self.take_ready_shader_source_validation(&validation_key)
                .expect("installed Shadow module must consume its validation artifact");
            self.record_shader_module_creation(target, creation_elapsed);
        }
        if !self.shadow_mesh_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shadow mesh shader module cached");
            let creation_started = std::time::Instant::now();
            let pipeline = create_shadow_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                kind,
                key,
                self.runtime_pipeline_cache.cache(),
            );
            let creation_elapsed = creation_started.elapsed();
            self.shadow_mesh_pipelines.insert(variant_id, pipeline);
            self.bind_pipeline_shader_module_reference(target, variant_id, &shader_key);
            self.record_render_pipeline_creation(target, creation_elapsed);
        }
        let pipeline_validation_failed = self.track_pipeline_creation_error_scope(
            shader_variant_key,
            target,
            variant_id,
            shader_key,
            error_scope,
        );
        if pipeline_validation_failed {
            self.drain_pipeline_creation_diagnostics();
            self.mark_pipeline_failure_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
                "Shadow pipeline WGPU validation failed",
            );
            return self.unavailable_pipeline_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
            );
        }
        self.clear_pipeline_unavailable_state_for_target(target, variant_id);
        PipelineAdmission::Ready(())
    }

    pub(crate) fn ensure_shadow_pipeline_admission_for_variant(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        expected_kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        let Some(target) = shadow_pipeline_target(expected_kind) else {
            return PipelineAdmission::unavailable(
                PipelineAdmissionReason::WrongPass,
                std::time::Duration::ZERO,
            );
        };
        if self.shadow_mesh_pipelines.contains_key(&variant_id) {
            self.clear_pipeline_unavailable_state_for_target(target, variant_id);
            return PipelineAdmission::Ready(());
        }
        if let Some(reason) = self.pipeline_failure_reason_for_target(target, variant_id) {
            return self.unavailable_pipeline_for_target(target, variant_id, reason);
        }
        let Some((kind, pipeline_key, shader_variant_key)) =
            self.pipeline_and_shader_key_for_variant(variant_id)
        else {
            return self.unavailable_pipeline_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::UnknownVariant,
            );
        };
        if kind != expected_kind {
            return self.unavailable_pipeline_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::WrongPass,
            );
        }
        self.ensure_shadow_pipeline(
            device,
            streamer,
            variant_id,
            &pipeline_key,
            kind,
            &shader_variant_key,
        )
    }

    pub(crate) fn shadow_pipeline_for_ready_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> &wgpu::RenderPipeline {
        self.shadow_mesh_pipelines
            .get(&variant_id)
            .expect("Ready Shadow pipeline admission must retain its pipeline")
    }
}

const fn shadow_pipeline_target(kind: MeshPassPipelineKind) -> Option<PipelineCreationTarget> {
    match kind {
        MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => {
            Some(PipelineCreationTarget::MeshPass(kind))
        }
        _ => None,
    }
}

fn shadow_mesh_shader_key(variant_key: &ShaderVariantKey, source_hash: &str) -> String {
    format!(
        "{}#{}#{}",
        SHADOW_MESH_SHADER_KEY_PREFIX,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPassType;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;

    use super::super::PipelineCreationTarget;
    use super::super::mesh_pipeline_shadow_template_source_for_geometry;
    use super::{SHADOW_MESH_SHADER_KEY_PREFIX, shadow_mesh_shader_key, shadow_pipeline_target};

    #[test]
    fn shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash() {
        let variant_key =
            default_pipeline_key().shader_variant_key(ShaderPassType::Shadow, "wgpu-runtime");
        let source = match mesh_pipeline_shadow_template_source_for_geometry(
            &default_pipeline_key(),
            variant_key.geometry_source,
        ) {
            Ok(source) => source,
            Err(error) => panic!("shadow template source assembly failed: {error:?}"),
        };
        let key = shadow_mesh_shader_key(&variant_key, &source.source_hash);

        assert!(key.starts_with(SHADOW_MESH_SHADER_KEY_PREFIX));
        assert!(key.contains(&variant_key.canonical_string()));
        assert!(key.contains(&source.source_hash));
    }

    #[test]
    fn shadow_pipeline_target_preserves_alpha_mask_identity() {
        assert_eq!(
            shadow_pipeline_target(MeshPassPipelineKind::ShadowDepth),
            Some(PipelineCreationTarget::MeshPass(
                MeshPassPipelineKind::ShadowDepth
            ))
        );
        assert_eq!(
            shadow_pipeline_target(MeshPassPipelineKind::ShadowDepthAlphaMask),
            Some(PipelineCreationTarget::MeshPass(
                MeshPassPipelineKind::ShadowDepthAlphaMask
            ))
        );
        assert_eq!(shadow_pipeline_target(MeshPassPipelineKind::Base), None);
    }
}
