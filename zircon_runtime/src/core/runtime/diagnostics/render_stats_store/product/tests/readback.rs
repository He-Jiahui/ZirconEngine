use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::render::RenderStats;

use super::super::record;
use super::assert_series;

#[test]
fn render_product_diagnostics_record_shared_readback_queue_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_readback_in_flight_count: 3,
        last_readback_bytes: 512,
        last_readback_completed_count: 2,
        last_readback_completed_bytes: 256,
        last_readback_slot_reuse_rejection_count: 1,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.readback.in_flight_count", 3.0, "count");
    assert_series(&store, "render.readback.in_flight_bytes", 512.0, "bytes");
    assert_series(
        &store,
        "render.readback.completed_request_count",
        2.0,
        "count",
    );
    assert_series(&store, "render.readback.completed_bytes", 256.0, "bytes");
    assert_series(
        &store,
        "render.readback.slot_reuse_rejection_count",
        1.0,
        "count",
    );
}
