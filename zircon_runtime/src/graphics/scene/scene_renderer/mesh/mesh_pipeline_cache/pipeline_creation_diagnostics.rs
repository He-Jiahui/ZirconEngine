use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::pipeline::PipelineAdmissionReason;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::mesh_pipeline_cache::{
    MeshPipelineCache, PipelineCreationTarget, MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS,
};

/// 保存创建点已解析的 WGPU 校验结果，供有界诊断队列随后按目标驱逐失败管线。
pub(super) struct PendingPipelineCreationDiagnostic {
    shader_variant_key: ShaderVariantKey,
    target: PipelineCreationTarget,
    variant_id: MeshPipelineVariantId,
    shader_key: String,
    error: Option<String>,
}

impl MeshPipelineCache {
    pub(crate) fn drain_pipeline_creation_diagnostics(&mut self) {
        self.drain_shader_source_validation_diagnostics();
        self.consume_resolved_pipeline_creation_diagnostics();
    }

    pub(crate) fn finish_pipeline_creation_diagnostics_for_variant(
        &mut self,
        key: &ShaderVariantKey,
    ) -> Result<bool, String> {
        let mut pending = Vec::with_capacity(self.pending_pipeline_creation_diagnostics.len());
        let mut matched_scope_count = 0;
        let mut matched = Vec::new();
        for diagnostic in self.pending_pipeline_creation_diagnostics.drain(..) {
            if diagnostic.shader_variant_key != *key {
                pending.push(diagnostic);
                continue;
            }
            matched_scope_count += 1;
            matched.push(diagnostic);
        }
        self.pending_pipeline_creation_diagnostics = pending;
        let messages = matched
            .into_iter()
            .filter_map(|diagnostic| self.consume_resolved_pipeline_creation_diagnostic(diagnostic))
            .collect::<Vec<_>>();
        if messages.is_empty() {
            Ok(matched_scope_count != 0)
        } else {
            Err(messages.join("; "))
        }
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn track_pipeline_creation_error_scope(
        &mut self,
        key: &ShaderVariantKey,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        shader_key: String,
        error_scope: wgpu::ErrorScopeGuard,
    ) -> bool {
        let error = {
            crate::profile_scope!("render", "shader_pipeline", "wgpu_pipeline_error_scope_pop");
            pollster::block_on(error_scope.pop()).map(|error| error.to_string())
        };
        let failed = error.is_some();
        if self.pending_pipeline_creation_diagnostics.len()
            >= MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS
        {
            crate::profile_counter!("render", "mesh_pipeline_diagnostic_rollover", 1);
            self.consume_resolved_pipeline_creation_diagnostics();
        }
        debug_assert!(
            self.pending_pipeline_creation_diagnostics.len()
                < MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS
        );
        self.pending_pipeline_creation_diagnostics
            .push(PendingPipelineCreationDiagnostic {
                shader_variant_key: key.clone(),
                target,
                variant_id,
                shader_key,
                error,
            });
        crate::profile_counter!(
            "render",
            "mesh_pipeline_diagnostic_queue_depth",
            self.pending_pipeline_creation_diagnostics.len()
        );
        failed
    }

    fn consume_resolved_pipeline_creation_diagnostics(&mut self) {
        let mut diagnostics = std::mem::take(&mut self.pending_pipeline_creation_diagnostics);
        for diagnostic in diagnostics.drain(..) {
            let _ = self.consume_resolved_pipeline_creation_diagnostic(diagnostic);
        }
        self.pending_pipeline_creation_diagnostics = diagnostics;
        crate::profile_counter!("render", "mesh_pipeline_diagnostic_queue_depth", 0);
    }

    fn consume_resolved_pipeline_creation_diagnostic(
        &mut self,
        diagnostic: PendingPipelineCreationDiagnostic,
    ) -> Option<String> {
        let PendingPipelineCreationDiagnostic {
            shader_variant_key,
            target,
            variant_id,
            shader_key,
            error,
        } = diagnostic;
        let error = error?;
        self.mark_pipeline_failure_for_target(
            target,
            variant_id,
            PipelineAdmissionReason::PipelineValidationFailed,
            error.clone(),
        );
        self.invalidate_pipeline_creation_target(&target, &variant_id, &shader_key);
        self.record_shader_variant_pipeline_creation_message(&shader_variant_key, error.clone());
        Some(error)
    }

    fn invalidate_pipeline_creation_target(
        &mut self,
        target: &PipelineCreationTarget,
        variant_id: &MeshPipelineVariantId,
        shader_key: &str,
    ) {
        let removed = match target {
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base) => {
                self.mesh_variant_pipelines.remove(variant_id).is_some()
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer) => {
                self.gbuffer_mesh_pipelines.remove(variant_id).is_some()
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass) => self
                .depth_prepass_mesh_pipelines
                .remove(variant_id)
                .is_some(),
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::HitProxy) => {
                self.hit_proxy_mesh_pipelines.remove(variant_id).is_some()
            }
            PipelineCreationTarget::MeshPass(
                MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask,
            ) => self.shadow_mesh_pipelines.remove(variant_id).is_some(),
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity) => {
                self.velocity_mesh_pipelines.remove(variant_id).is_some()
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMask) => self
                .taa_reactive_mask_mesh_pipelines
                .remove(variant_id)
                .is_some(),
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMaterialMask) => self
                .taa_reactive_material_mask_mesh_pipelines
                .remove(variant_id)
                .is_some(),
            PipelineCreationTarget::Oit => {
                self.oit_mesh_variant_pipelines.remove(variant_id).is_some()
            }
        };
        if removed {
            self.release_pipeline_shader_module_reference(*target, *variant_id, shader_key);
        }
    }
}

#[cfg(test)]
#[path = "tests/pipeline_creation_diagnostics.rs"]
mod tests;
