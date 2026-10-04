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
    metrics.record_shader_source_validation_queue_result(PipelineAsyncQueueResult::AlreadyPending);
    metrics.record_shader_source_validation_queue_result(PipelineAsyncQueueResult::Full);
    metrics
        .record_shader_source_validation_queue_result(PipelineAsyncQueueResult::WorkerUnavailable);
    metrics.record_shader_source_validation_started(
        "same-source-contract",
        std::time::Duration::from_micros(7),
    );
    metrics.record_shader_source_validation_completed(std::time::Duration::from_micros(11), true);
    metrics.record_shader_source_validation_started(
        "same-source-contract",
        std::time::Duration::from_micros(13),
    );
    metrics.record_shader_source_validation_completed(std::time::Duration::from_micros(17), false);

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
