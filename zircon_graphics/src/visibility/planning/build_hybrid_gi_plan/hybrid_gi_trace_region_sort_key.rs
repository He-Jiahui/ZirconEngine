use std::cmp::Ordering;

use zircon_scene::RenderHybridGiTraceRegion;

pub(super) fn hybrid_gi_trace_region_sort_key(
    left: &RenderHybridGiTraceRegion,
    right: &RenderHybridGiTraceRegion,
) -> Ordering {
    right
        .screen_coverage
        .partial_cmp(&left.screen_coverage)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.region_id.cmp(&right.region_id))
}
