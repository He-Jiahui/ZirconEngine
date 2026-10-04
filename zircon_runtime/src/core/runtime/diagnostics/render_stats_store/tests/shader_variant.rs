use crate::core::framework::render::{
    RenderStats, ShaderPipelineTarget, ShaderPipelineTargetMetrics,
};
use crate::core::runtime::diagnostics::DiagnosticStore;

use super::{record, PIPELINE_TARGET_DIAGNOSTIC_PATHS};

#[test]
fn pipeline_target_diagnostic_paths_follow_public_target_order() {
    for (target, paths) in ShaderPipelineTarget::ALL
        .into_iter()
        .zip(PIPELINE_TARGET_DIAGNOSTIC_PATHS.iter())
    {
        assert_eq!(paths.target, target);
        assert!(paths
            .registered_pipeline_variant_count
            .contains(target.token()));
        assert!(paths.unique_shader_source_count.contains(target.token()));
        assert!(paths
            .render_pipeline_creation_count
            .contains(target.token()));
        assert!(paths.shader_module_creation_count.contains(target.token()));
        assert!(paths
            .render_pipeline_creation_cpu_microseconds
            .contains(target.token()));
        assert!(paths
            .shader_module_creation_cpu_microseconds
            .contains(target.token()));
    }
}

#[test]
fn shader_variant_diagnostics_record_registered_pipeline_expansion_gauges() {
    let mut store = DiagnosticStore::default();
    let mut stats = RenderStats {
        submitted_frames: 12,
        ..RenderStats::default()
    };
    stats
        .last_shader_variant_miss_report
        .record_registered_variant_counts(16, 1, 1);
    stats
        .last_shader_variant_miss_report
        .record_cached_gpu_object_counts(12, 3);
    stats
        .last_shader_variant_miss_report
        .record_gpu_object_creation_totals(9, 4, 42, 17);
    stats
        .last_shader_variant_miss_report
        .record_async_base_pipeline_queue_wait_totals(3, 88);
    stats
        .last_shader_variant_miss_report
        .record_shader_source_validation_metrics(
            crate::core::framework::render::ShaderSourceValidationMetrics {
                queued_count: 5,
                job_count: 5,
                unique_source_count: 3,
                duplicate_job_count: 2,
                success_count: 4,
                failure_count: 1,
                queue_wait_microseconds: 31,
                validation_cpu_microseconds: 47,
                ..Default::default()
            },
        );
    stats
        .last_shader_variant_miss_report
        .record_registered_pipeline_target_variant_count(ShaderPipelineTarget::ShadowDepth, 5);
    stats
        .last_shader_variant_miss_report
        .record_pipeline_target_runtime_metrics(
            ShaderPipelineTarget::ShadowDepth,
            ShaderPipelineTargetMetrics {
                unique_shader_source_count: 2,
                render_pipeline_creation_count: 4,
                shader_module_creation_count: 3,
                render_pipeline_creation_cpu_microseconds: 71,
                shader_module_creation_cpu_microseconds: 53,
                ..ShaderPipelineTargetMetrics::default()
            },
        );

    record(&mut store, &stats);

    let snapshot = store.snapshot();
    assert_series(
        &snapshot,
        "render.shader_variant.registered_pipeline_variant_count",
        16.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.registered_shader_variant_count",
        1.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.texture_presence_normalized_pipeline_variant_count",
        1.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.texture_presence_equivalent_pipeline_variant_count",
        15.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.cached_render_pipeline_count",
        12.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.cached_shader_module_count",
        3.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.render_pipeline_creation_count",
        9.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.shader_module_creation_count",
        4.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.render_pipeline_creation_cpu_microseconds",
        42.0,
        "microseconds",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.shader_module_creation_cpu_microseconds",
        17.0,
        "microseconds",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.async_base_pipeline_queue_wait_count",
        3.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.async_base_pipeline_queue_wait_microseconds",
        88.0,
        "microseconds",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.source_validation.job_count",
        5.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.source_validation.duplicate_job_count",
        2.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.source_validation.validation_cpu_microseconds",
        47.0,
        "microseconds",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.registered_pipeline_variant_count",
        5.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.unique_shader_source_count",
        2.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.render_pipeline_creation_count",
        4.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.shader_module_creation_count",
        3.0,
        "count",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.render_pipeline_creation_cpu_microseconds",
        71.0,
        "microseconds",
    );
    assert_series(
        &snapshot,
        "render.shader_variant.target.shadow_depth.shader_module_creation_cpu_microseconds",
        53.0,
        "microseconds",
    );
}

fn assert_series(
    snapshot: &crate::core::runtime::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    expected: f64,
    expected_unit: &str,
) {
    let series = snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series {path}"));
    assert_eq!(series.current, Some(expected));
    assert_eq!(series.unit.as_deref(), Some(expected_unit));
}
