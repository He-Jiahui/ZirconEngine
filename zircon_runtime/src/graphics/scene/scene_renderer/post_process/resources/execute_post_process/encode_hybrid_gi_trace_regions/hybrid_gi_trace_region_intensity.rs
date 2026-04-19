use crate::core::framework::render::RenderHybridGiTraceRegion;

pub(super) fn hybrid_gi_trace_region_intensity(
    region: &RenderHybridGiTraceRegion,
    tracing_budget: u32,
) -> f32 {
    let budget_weight = (tracing_budget.max(1) as f32 / 4.0).clamp(0.25, 1.0);
    let coverage_weight = (0.4 + region.screen_coverage.clamp(0.0, 1.0) * 0.6).clamp(0.25, 1.0);
    (budget_weight * coverage_weight).clamp(0.2, 1.0)
}
