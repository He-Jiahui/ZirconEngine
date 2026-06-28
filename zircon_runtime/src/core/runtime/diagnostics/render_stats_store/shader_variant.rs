use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};

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
}
