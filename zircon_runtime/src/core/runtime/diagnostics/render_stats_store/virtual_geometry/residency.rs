use crate::core::framework::render::RenderStats;

use super::super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.requested_page_count",
        frame_index,
        stats.last_virtual_geometry_requested_page_count,
        &["render", "virtual_geometry", "page", "request"],
    );
    record_count(
        store,
        "render.virtual_geometry.dirty_page_count",
        frame_index,
        stats.last_virtual_geometry_dirty_page_count,
        &["render", "virtual_geometry", "page", "dirty"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_table_entry_count",
        frame_index,
        stats.last_virtual_geometry_page_table_entry_count,
        &["render", "virtual_geometry", "page_table"],
    );
    record_count(
        store,
        "render.virtual_geometry.resident_page_count",
        frame_index,
        stats.last_virtual_geometry_resident_page_count,
        &["render", "virtual_geometry", "page", "resident"],
    );
    record_count(
        store,
        "render.virtual_geometry.pending_request_count",
        frame_index,
        stats.last_virtual_geometry_pending_request_count,
        &["render", "virtual_geometry", "page", "pending"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_dependency_count",
        frame_index,
        stats.last_virtual_geometry_page_dependency_count,
        &["render", "virtual_geometry", "page", "dependency"],
    );
    record_count(
        store,
        "render.virtual_geometry.completed_page_count",
        frame_index,
        stats.last_virtual_geometry_completed_page_count,
        &["render", "virtual_geometry", "page", "completed"],
    );
    record_count(
        store,
        "render.virtual_geometry.replaced_page_count",
        frame_index,
        stats.last_virtual_geometry_replaced_page_count,
        &["render", "virtual_geometry", "page", "replacement"],
    );
}
