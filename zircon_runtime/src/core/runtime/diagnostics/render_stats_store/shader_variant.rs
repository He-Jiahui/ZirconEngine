use crate::core::framework::render::{
    RenderStats, ShaderPipelineTarget, SHADER_PIPELINE_TARGET_COUNT,
};

use super::{record_count, record_microseconds, DiagnosticStore};

struct PipelineTargetDiagnosticPaths {
    target: ShaderPipelineTarget,
    registered_pipeline_variant_count: &'static str,
    unique_shader_source_count: &'static str,
    render_pipeline_creation_count: &'static str,
    shader_module_creation_count: &'static str,
    render_pipeline_creation_cpu_microseconds: &'static str,
    shader_module_creation_cpu_microseconds: &'static str,
}

macro_rules! pipeline_target_diagnostic_paths {
    ($target:ident, $token:literal) => {
        PipelineTargetDiagnosticPaths {
            target: ShaderPipelineTarget::$target,
            registered_pipeline_variant_count: concat!(
                "render.shader_variant.target.",
                $token,
                ".registered_pipeline_variant_count"
            ),
            unique_shader_source_count: concat!(
                "render.shader_variant.target.",
                $token,
                ".unique_shader_source_count"
            ),
            render_pipeline_creation_count: concat!(
                "render.shader_variant.target.",
                $token,
                ".render_pipeline_creation_count"
            ),
            shader_module_creation_count: concat!(
                "render.shader_variant.target.",
                $token,
                ".shader_module_creation_count"
            ),
            render_pipeline_creation_cpu_microseconds: concat!(
                "render.shader_variant.target.",
                $token,
                ".render_pipeline_creation_cpu_microseconds"
            ),
            shader_module_creation_cpu_microseconds: concat!(
                "render.shader_variant.target.",
                $token,
                ".shader_module_creation_cpu_microseconds"
            ),
        }
    };
}

const PIPELINE_TARGET_DIAGNOSTIC_PATHS: [PipelineTargetDiagnosticPaths;
    SHADER_PIPELINE_TARGET_COUNT] = [
    pipeline_target_diagnostic_paths!(Base, "base"),
    pipeline_target_diagnostic_paths!(GBuffer, "gbuffer"),
    pipeline_target_diagnostic_paths!(DepthPrepass, "depth_prepass"),
    pipeline_target_diagnostic_paths!(HitProxy, "hit_proxy"),
    pipeline_target_diagnostic_paths!(ShadowDepth, "shadow_depth"),
    pipeline_target_diagnostic_paths!(ShadowDepthAlphaMask, "shadow_depth_alpha_mask"),
    pipeline_target_diagnostic_paths!(Velocity, "velocity"),
    pipeline_target_diagnostic_paths!(TaaReactiveMask, "taa_reactive_mask"),
    pipeline_target_diagnostic_paths!(TaaReactiveMaterialMask, "taa_reactive_material_mask"),
    pipeline_target_diagnostic_paths!(Oit, "oit"),
];

const PIPELINE_TARGET_DIAGNOSTIC_TAGS: &[&str] =
    &["render", "shader", "variant", "pipeline", "target"];

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = &stats.last_shader_variant_miss_report;
    record_count(
        store,
        "render.shader_variant.request_count",
        frame_index,
        report.request_count,
        &["render", "shader", "variant"],
    );
    record_count(
        store,
        "render.shader_variant.memory_hit_count",
        frame_index,
        report.memory_hit_count,
        &["render", "shader", "variant", "cache", "memory"],
    );
    record_count(
        store,
        "render.shader_variant.disk_hit_count",
        frame_index,
        report.disk_hit_count,
        &["render", "shader", "variant", "cache", "disk"],
    );
    record_count(
        store,
        "render.shader_variant.compile_miss_count",
        frame_index,
        report.compile_miss_count,
        &["render", "shader", "variant", "compile"],
    );
    record_count(
        store,
        "render.shader_variant.disk_write_count",
        frame_index,
        report.disk_write_count,
        &["render", "shader", "variant", "cache", "disk", "write"],
    );
    record_count(
        store,
        "render.shader_variant.disk_error_count",
        frame_index,
        report.disk_error_count,
        &["render", "shader", "variant", "cache", "disk", "error"],
    );
    record_count(
        store,
        "render.shader_variant.registered_pipeline_variant_count",
        frame_index,
        report.registered_pipeline_variant_count,
        &["render", "shader", "variant", "pipeline", "registered"],
    );
    record_count(
        store,
        "render.shader_variant.registered_shader_variant_count",
        frame_index,
        report.registered_shader_variant_count,
        &["render", "shader", "variant", "registered"],
    );
    record_count(
        store,
        "render.shader_variant.texture_presence_normalized_pipeline_variant_count",
        frame_index,
        report.texture_presence_normalized_pipeline_variant_count,
        &["render", "shader", "variant", "pipeline", "normalized"],
    );
    record_count(
        store,
        "render.shader_variant.texture_presence_equivalent_pipeline_variant_count",
        frame_index,
        report.texture_presence_equivalent_pipeline_variant_count,
        &[
            "render",
            "shader",
            "variant",
            "pipeline",
            "texture_presence",
        ],
    );
    record_count(
        store,
        "render.shader_variant.cached_render_pipeline_count",
        frame_index,
        report.cached_render_pipeline_count,
        &["render", "shader", "variant", "pipeline", "cache"],
    );
    record_count(
        store,
        "render.shader_variant.cached_shader_module_count",
        frame_index,
        report.cached_shader_module_count,
        &["render", "shader", "variant", "module", "cache"],
    );
    record_count(
        store,
        "render.shader_variant.render_pipeline_creation_count",
        frame_index,
        report.render_pipeline_creation_count,
        &["render", "shader", "variant", "pipeline", "creation"],
    );
    record_count(
        store,
        "render.shader_variant.shader_module_creation_count",
        frame_index,
        report.shader_module_creation_count,
        &["render", "shader", "variant", "module", "creation"],
    );
    record_microseconds(
        store,
        "render.shader_variant.render_pipeline_creation_cpu_microseconds",
        frame_index,
        report.render_pipeline_creation_cpu_microseconds,
        &["render", "shader", "variant", "pipeline", "creation", "cpu"],
    );
    record_microseconds(
        store,
        "render.shader_variant.shader_module_creation_cpu_microseconds",
        frame_index,
        report.shader_module_creation_cpu_microseconds,
        &["render", "shader", "variant", "module", "creation", "cpu"],
    );
    record_count(
        store,
        "render.shader_variant.async_base_pipeline_queue_wait_count",
        frame_index,
        report.async_base_pipeline_queue_wait_count,
        &["render", "shader", "variant", "pipeline", "async", "queue"],
    );
    record_microseconds(
        store,
        "render.shader_variant.async_base_pipeline_queue_wait_microseconds",
        frame_index,
        report.async_base_pipeline_queue_wait_microseconds,
        &["render", "shader", "variant", "pipeline", "async", "queue"],
    );
    let validation = report.shader_source_validation_metrics;
    let validation_tags = &["render", "shader", "source", "validation"];
    record_count(
        store,
        "render.shader_variant.source_validation.queued_count",
        frame_index,
        validation.queued_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.already_pending_count",
        frame_index,
        validation.already_pending_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.full_count",
        frame_index,
        validation.full_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.worker_unavailable_count",
        frame_index,
        validation.worker_unavailable_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.job_count",
        frame_index,
        validation.job_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.unique_source_count",
        frame_index,
        validation.unique_source_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.duplicate_job_count",
        frame_index,
        validation.duplicate_job_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.success_count",
        frame_index,
        validation.success_count,
        validation_tags,
    );
    record_count(
        store,
        "render.shader_variant.source_validation.failure_count",
        frame_index,
        validation.failure_count,
        validation_tags,
    );
    record_microseconds(
        store,
        "render.shader_variant.source_validation.queue_wait_microseconds",
        frame_index,
        validation.queue_wait_microseconds,
        validation_tags,
    );
    record_microseconds(
        store,
        "render.shader_variant.source_validation.validation_cpu_microseconds",
        frame_index,
        validation.validation_cpu_microseconds,
        validation_tags,
    );
    for paths in PIPELINE_TARGET_DIAGNOSTIC_PATHS {
        let metrics = report.pipeline_target_metrics(paths.target);
        record_count(
            store,
            paths.registered_pipeline_variant_count,
            frame_index,
            metrics.registered_pipeline_variant_count,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
        record_count(
            store,
            paths.unique_shader_source_count,
            frame_index,
            metrics.unique_shader_source_count,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
        record_count(
            store,
            paths.render_pipeline_creation_count,
            frame_index,
            metrics.render_pipeline_creation_count,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
        record_count(
            store,
            paths.shader_module_creation_count,
            frame_index,
            metrics.shader_module_creation_count,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
        record_microseconds(
            store,
            paths.render_pipeline_creation_cpu_microseconds,
            frame_index,
            metrics.render_pipeline_creation_cpu_microseconds,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
        record_microseconds(
            store,
            paths.shader_module_creation_cpu_microseconds,
            frame_index,
            metrics.shader_module_creation_cpu_microseconds,
            PIPELINE_TARGET_DIAGNOSTIC_TAGS,
        );
    }
    record_count(
        store,
        "render.shader_variant.pipeline_diagnostic_count",
        frame_index,
        report.pipeline_diagnostics().len(),
        &["render", "shader", "variant", "pipeline", "diagnostic"],
    );
}

#[cfg(test)]
mod tests {
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
}
