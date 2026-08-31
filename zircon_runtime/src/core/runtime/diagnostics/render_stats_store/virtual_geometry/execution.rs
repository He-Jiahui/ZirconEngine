use crate::core::framework::render::RenderStats;

use super::super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.indirect_draw_count",
        frame_index,
        stats.last_virtual_geometry_indirect_draw_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_buffer_count",
        frame_index,
        stats.last_virtual_geometry_indirect_buffer_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_args_count",
        frame_index,
        stats.last_virtual_geometry_indirect_args_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_segment_count",
        frame_index,
        stats.last_virtual_geometry_indirect_segment_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_segment_count,
        &["render", "virtual_geometry", "execution"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_page_count",
        frame_index,
        stats.last_virtual_geometry_execution_page_count,
        &["render", "virtual_geometry", "execution", "page"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_resident_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_resident_segment_count,
        &["render", "virtual_geometry", "execution", "resident"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_pending_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_pending_segment_count,
        &["render", "virtual_geometry", "execution", "pending"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_missing_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_missing_segment_count,
        &["render", "virtual_geometry", "execution", "missing"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_repeated_draw_count",
        frame_index,
        stats.last_virtual_geometry_execution_repeated_draw_count,
        &["render", "virtual_geometry", "execution", "repeat"],
    );
}
