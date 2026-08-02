use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.material.count",
        frame_index,
        stats.last_material_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.ready_count",
        frame_index,
        stats.last_material_ready_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.fallback_count",
        frame_index,
        stats.last_material_fallback_count,
        &["render", "material", "fallback"],
    );
    record_count(
        store,
        "render.material.validation_error_count",
        frame_index,
        stats.last_material_validation_error_count,
        &["render", "material", "validation"],
    );
    record_count(
        store,
        "render.material.diagnostic_count",
        frame_index,
        stats.last_material_diagnostic_count,
        &["render", "material", "diagnostic"],
    );
}
