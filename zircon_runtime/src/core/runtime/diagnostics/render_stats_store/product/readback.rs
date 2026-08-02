use crate::core::framework::render::RenderStats;

use super::{record_bytes, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.readback.in_flight_count",
        frame_index,
        stats.last_readback_in_flight_count,
        &["render", "readback"],
    );
    record_bytes(
        store,
        "render.readback.in_flight_bytes",
        frame_index,
        stats.last_readback_bytes,
        &["render", "readback"],
    );
    record_count(
        store,
        "render.readback.completed_request_count",
        frame_index,
        stats.last_readback_completed_count,
        &["render", "readback"],
    );
    record_bytes(
        store,
        "render.readback.completed_bytes",
        frame_index,
        stats.last_readback_completed_bytes,
        &["render", "readback"],
    );
    record_count(
        store,
        "render.readback.slot_reuse_rejection_count",
        frame_index,
        stats.last_readback_slot_reuse_rejection_count as usize,
        &["render", "readback"],
    );
}
