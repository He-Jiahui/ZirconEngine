use std::cmp::Ordering;

use super::super::sources::HybridGiVisibilityPlanTraceRegion;

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_trace_region_sort_key(
    left: &HybridGiVisibilityPlanTraceRegion,
    right: &HybridGiVisibilityPlanTraceRegion,
) -> Ordering {
    right
        .screen_coverage
        .partial_cmp(&left.screen_coverage)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.region_id.cmp(&right.region_id))
}
