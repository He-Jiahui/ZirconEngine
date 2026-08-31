use std::collections::HashSet;
use std::sync::Mutex;

use crate::core::framework::render::{
    SHADER_PIPELINE_TARGET_COUNT, ShaderPipelineTarget, ShaderPipelineTargetMetrics,
    ShaderSourceValidationMetrics,
};
use crate::graphics::pipeline::PipelineAsyncQueueResult;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;

use super::mesh_pipeline_cache::PipelineCreationTarget;

#[derive(Default)]
pub(super) struct MeshPipelineCreationMetrics {
    state: Mutex<MeshPipelineCreationMetricsState>,
}

#[derive(Default)]
struct MeshPipelineCreationMetricsState {
    snapshot: MeshPipelineCreationMetricsSnapshot,
    observed_shader_sources: [HashSet<String>; SHADER_PIPELINE_TARGET_COUNT],
    observed_shader_validation_sources: HashSet<String>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct MeshPipelineCreationMetricsSnapshot {
    pub(super) render_pipeline_creation_count: usize,
    pub(super) shader_module_creation_count: usize,
    pub(super) render_pipeline_creation_cpu_microseconds: u64,
    pub(super) shader_module_creation_cpu_microseconds: u64,
    pub(super) async_base_pipeline_queue_wait_count: usize,
    pub(super) async_base_pipeline_queue_wait_microseconds: u64,
    pub(super) shader_source_validation: ShaderSourceValidationMetrics,
    pub(super) pipeline_targets: [ShaderPipelineTargetMetrics; SHADER_PIPELINE_TARGET_COUNT],
}

impl MeshPipelineCreationMetrics {
    pub(super) fn record_observed_shader_source(
        &self,
        target: PipelineCreationTarget,
        source_hash: &str,
    ) {
        let target = shader_pipeline_target(target);
        let mut state = self.lock_state();
        let unique_shader_source_count = {
            let observed = &mut state.observed_shader_sources[target.index()];
            if observed.contains(source_hash) {
                return;
            }
            observed.insert(source_hash.to_owned());
            observed.len()
        };
        state.snapshot.pipeline_targets[target.index()].unique_shader_source_count =
            unique_shader_source_count;
    }

    pub(super) fn record_render_pipeline_creation(
        &self,
        target: PipelineCreationTarget,
        elapsed: std::time::Duration,
    ) {
        let target = shader_pipeline_target(target);
        let elapsed = duration_microseconds_saturating(elapsed);
        let mut state = self.lock_state();
        state.snapshot.render_pipeline_creation_count = state
            .snapshot
            .render_pipeline_creation_count
            .saturating_add(1);
        state.snapshot.render_pipeline_creation_cpu_microseconds = state
            .snapshot
            .render_pipeline_creation_cpu_microseconds
            .saturating_add(elapsed);
        let metrics = &mut state.snapshot.pipeline_targets[target.index()];
        metrics.render_pipeline_creation_count =
            metrics.render_pipeline_creation_count.saturating_add(1);
        metrics.render_pipeline_creation_cpu_microseconds = metrics
            .render_pipeline_creation_cpu_microseconds
            .saturating_add(elapsed);
    }

    pub(super) fn record_shader_module_creation(
        &self,
        target: PipelineCreationTarget,
        elapsed: std::time::Duration,
    ) {
        let target = shader_pipeline_target(target);
        let elapsed = duration_microseconds_saturating(elapsed);
        let mut state = self.lock_state();
        state.snapshot.shader_module_creation_count = state
            .snapshot
            .shader_module_creation_count
            .saturating_add(1);
        state.snapshot.shader_module_creation_cpu_microseconds = state
            .snapshot
            .shader_module_creation_cpu_microseconds
            .saturating_add(elapsed);
        let metrics = &mut state.snapshot.pipeline_targets[target.index()];
        metrics.shader_module_creation_count =
            metrics.shader_module_creation_count.saturating_add(1);
        metrics.shader_module_creation_cpu_microseconds = metrics
            .shader_module_creation_cpu_microseconds
            .saturating_add(elapsed);
    }

    pub(super) fn record_async_base_pipeline_queue_wait(&self, elapsed: std::time::Duration) {
        let mut state = self.lock_state();
        state.snapshot.async_base_pipeline_queue_wait_count = state
            .snapshot
            .async_base_pipeline_queue_wait_count
            .saturating_add(1);
        state.snapshot.async_base_pipeline_queue_wait_microseconds = state
            .snapshot
            .async_base_pipeline_queue_wait_microseconds
            .saturating_add(duration_microseconds_saturating(elapsed));
    }

    pub(super) fn record_shader_source_validation_queue_result(
        &self,
        result: PipelineAsyncQueueResult,
    ) {
        let mut state = self.lock_state();
        let metrics = &mut state.snapshot.shader_source_validation;
        match result {
            PipelineAsyncQueueResult::Queued => {
                metrics.queued_count = metrics.queued_count.saturating_add(1)
            }
            PipelineAsyncQueueResult::AlreadyPending => {
                metrics.already_pending_count = metrics.already_pending_count.saturating_add(1)
            }
            PipelineAsyncQueueResult::Full => {
                metrics.full_count = metrics.full_count.saturating_add(1)
            }
            PipelineAsyncQueueResult::WorkerUnavailable => {
                metrics.worker_unavailable_count =
                    metrics.worker_unavailable_count.saturating_add(1)
            }
        }
    }

    pub(super) fn record_shader_source_validation_started(
        &self,
        source_identity: &str,
        queue_wait: std::time::Duration,
    ) {
        let mut state = self.lock_state();
        let is_new_source = if state
            .observed_shader_validation_sources
            .contains(source_identity)
        {
            false
        } else {
            state
                .observed_shader_validation_sources
                .insert(source_identity.to_owned());
            true
        };
        let unique_source_count = state.observed_shader_validation_sources.len();
        let metrics = &mut state.snapshot.shader_source_validation;
        metrics.job_count = metrics.job_count.saturating_add(1);
        metrics.queue_wait_microseconds = metrics
            .queue_wait_microseconds
            .saturating_add(duration_microseconds_saturating(queue_wait));
        metrics.unique_source_count = unique_source_count;
        if !is_new_source {
            metrics.duplicate_job_count = metrics.duplicate_job_count.saturating_add(1);
        }
    }

    pub(super) fn record_shader_source_validation_completed(
        &self,
        elapsed: std::time::Duration,
        succeeded: bool,
    ) {
        let mut state = self.lock_state();
        let metrics = &mut state.snapshot.shader_source_validation;
        metrics.validation_cpu_microseconds = metrics
            .validation_cpu_microseconds
            .saturating_add(duration_microseconds_saturating(elapsed));
        if succeeded {
            metrics.success_count = metrics.success_count.saturating_add(1);
        } else {
            metrics.failure_count = metrics.failure_count.saturating_add(1);
        }
    }

    pub(super) fn snapshot(&self) -> MeshPipelineCreationMetricsSnapshot {
        self.lock_state().snapshot
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, MeshPipelineCreationMetricsState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub(super) const fn shader_pipeline_target(target: PipelineCreationTarget) -> ShaderPipelineTarget {
    match target {
        PipelineCreationTarget::MeshPass(kind) => shader_pipeline_target_for_mesh_kind(kind),
        PipelineCreationTarget::Oit => ShaderPipelineTarget::Oit,
    }
}

pub(super) const fn shader_pipeline_target_for_mesh_kind(
    kind: MeshPassPipelineKind,
) -> ShaderPipelineTarget {
    match kind {
        MeshPassPipelineKind::Base => ShaderPipelineTarget::Base,
        MeshPassPipelineKind::GBuffer => ShaderPipelineTarget::GBuffer,
        MeshPassPipelineKind::DepthPrepass => ShaderPipelineTarget::DepthPrepass,
        MeshPassPipelineKind::HitProxy => ShaderPipelineTarget::HitProxy,
        MeshPassPipelineKind::ShadowDepth => ShaderPipelineTarget::ShadowDepth,
        MeshPassPipelineKind::ShadowDepthAlphaMask => ShaderPipelineTarget::ShadowDepthAlphaMask,
        MeshPassPipelineKind::Velocity => ShaderPipelineTarget::Velocity,
        MeshPassPipelineKind::TaaReactiveMask => ShaderPipelineTarget::TaaReactiveMask,
        MeshPassPipelineKind::TaaReactiveMaterialMask => {
            ShaderPipelineTarget::TaaReactiveMaterialMask
        }
    }
}

fn duration_microseconds_saturating(duration: std::time::Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPipelineTarget;
    use crate::graphics::pipeline::PipelineAsyncQueueResult;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;

    use super::{MeshPipelineCreationMetrics, PipelineCreationTarget};

    #[test]
    fn target_metrics_deduplicate_sources_and_keep_creation_totals_exact() {
        let metrics = MeshPipelineCreationMetrics::default();
        let base = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
        let shadow = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepth);

        metrics.record_observed_shader_source(base, "shared-source");
        metrics.record_observed_shader_source(base, "shared-source");
        metrics.record_observed_shader_source(shadow, "shared-source");
        metrics.record_render_pipeline_creation(base, std::time::Duration::from_micros(23));
        metrics.record_render_pipeline_creation(shadow, std::time::Duration::from_micros(29));
        metrics.record_shader_module_creation(shadow, std::time::Duration::from_micros(17));
        metrics.record_async_base_pipeline_queue_wait(std::time::Duration::from_micros(11));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.render_pipeline_creation_count, 2);
        assert_eq!(snapshot.shader_module_creation_count, 1);
        assert_eq!(snapshot.render_pipeline_creation_cpu_microseconds, 52);
        assert_eq!(snapshot.shader_module_creation_cpu_microseconds, 17);
        assert_eq!(snapshot.async_base_pipeline_queue_wait_count, 1);
        assert_eq!(snapshot.async_base_pipeline_queue_wait_microseconds, 11);
        let base = snapshot.pipeline_targets[ShaderPipelineTarget::Base.index()];
        assert_eq!(base.unique_shader_source_count, 1);
        assert_eq!(base.render_pipeline_creation_count, 1);
        assert_eq!(base.render_pipeline_creation_cpu_microseconds, 23);
        let shadow = snapshot.pipeline_targets[ShaderPipelineTarget::ShadowDepth.index()];
        assert_eq!(shadow.unique_shader_source_count, 1);
        assert_eq!(shadow.render_pipeline_creation_count, 1);
        assert_eq!(shadow.shader_module_creation_count, 1);
    }

    #[test]
    fn source_validation_metrics_expose_duplicate_work_without_changing_identity() {
        let metrics = MeshPipelineCreationMetrics::default();
        metrics.record_shader_source_validation_queue_result(PipelineAsyncQueueResult::Queued);
        metrics
            .record_shader_source_validation_queue_result(PipelineAsyncQueueResult::AlreadyPending);
        metrics.record_shader_source_validation_queue_result(PipelineAsyncQueueResult::Full);
        metrics.record_shader_source_validation_queue_result(
            PipelineAsyncQueueResult::WorkerUnavailable,
        );
        metrics.record_shader_source_validation_started(
            "same-source-contract",
            std::time::Duration::from_micros(7),
        );
        metrics
            .record_shader_source_validation_completed(std::time::Duration::from_micros(11), true);
        metrics.record_shader_source_validation_started(
            "same-source-contract",
            std::time::Duration::from_micros(13),
        );
        metrics
            .record_shader_source_validation_completed(std::time::Duration::from_micros(17), false);

        let validation = metrics.snapshot().shader_source_validation;
        assert_eq!(validation.queued_count, 1);
        assert_eq!(validation.already_pending_count, 1);
        assert_eq!(validation.full_count, 1);
        assert_eq!(validation.worker_unavailable_count, 1);
        assert_eq!(validation.job_count, 2);
        assert_eq!(validation.unique_source_count, 1);
        assert_eq!(validation.duplicate_job_count, 1);
        assert_eq!(validation.success_count, 1);
        assert_eq!(validation.failure_count, 1);
        assert_eq!(validation.queue_wait_microseconds, 20);
        assert_eq!(validation.validation_cpu_microseconds, 28);
    }
}
