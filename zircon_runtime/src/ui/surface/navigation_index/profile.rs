use std::time::Instant;

use super::UiSurfaceNavigationIndex;

pub(super) fn record_navigation_rebuild_profile(
    index: &UiSurfaceNavigationIndex,
    rebuild_start: Instant,
) {
    crate::core::diagnostics::profiling::record_counter_batch(
        "runtime",
        &[
            ("ui.navigation_index.build_count", 1.0),
            (
                "ui.navigation_index.rebuild_elapsed_us",
                rebuild_start.elapsed().as_micros() as f64,
            ),
            ("ui.navigation_index.node_count", index.nodes.len() as f64),
            (
                "ui.navigation_index.candidate_count",
                index.spatial_all.len() as f64,
            ),
        ],
    );
}
