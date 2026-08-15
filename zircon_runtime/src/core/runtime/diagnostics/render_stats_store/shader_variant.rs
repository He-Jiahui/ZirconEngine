use crate::core::framework::render::RenderStats;

use super::{record_count, record_microseconds, DiagnosticStore};

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
    use crate::core::framework::render::RenderStats;
    use crate::core::runtime::diagnostics::DiagnosticStore;

    use super::record;

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
