use std::cmp::Ordering;

use zircon_scene::{RenderHybridGiProbe, RenderHybridGiTraceRegion};

const MIN_TRACE_SUPPORT_REACH: f32 = 0.0001;

pub(super) fn hybrid_gi_probe_request_sort_key(
    left: &RenderHybridGiProbe,
    right: &RenderHybridGiProbe,
    scheduled_trace_regions: &[RenderHybridGiTraceRegion],
) -> Ordering {
    probe_trace_support_score(right, scheduled_trace_regions)
        .total_cmp(&probe_trace_support_score(left, scheduled_trace_regions))
        .then_with(|| right.ray_budget.cmp(&left.ray_budget))
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}

fn probe_trace_support_score(
    probe: &RenderHybridGiProbe,
    scheduled_trace_regions: &[RenderHybridGiTraceRegion],
) -> f32 {
    scheduled_trace_regions
        .iter()
        .map(|region| {
            let reach = (region.bounds_radius + probe.radius).max(MIN_TRACE_SUPPORT_REACH);
            let distance_to_region = probe.position.distance(region.bounds_center);
            let falloff = (1.0 - distance_to_region / reach).max(0.0);
            falloff * falloff * region.screen_coverage.max(0.0)
        })
        .sum()
}
