use crate::core::framework::render::RenderHybridGiExtract;
use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};
use crate::graphics::hybrid_gi_extract_sources::{
    hybrid_gi_extract_probe_records, hybrid_gi_extract_trace_region_records,
    HybridGiExtractProbeRecord, HybridGiExtractTraceRegionRecord,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) struct HybridGiVisibilityPlanProbe
{
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) entity: EntityId,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) probe_id: u32,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) position: Vec3,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) radius: Real,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) parent_probe_id:
        Option<u32>,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) resident: bool,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) ray_budget: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) struct HybridGiVisibilityPlanTraceRegion
{
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) entity: EntityId,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) region_id: u32,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) bounds_center: Vec3,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) bounds_radius: Real,
    pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) screen_coverage: Real,
}

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_visibility_plan_probes(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiVisibilityPlanProbe> {
    hybrid_gi_extract_probe_records(extract)
        .into_iter()
        .map(HybridGiVisibilityPlanProbe::from)
        .collect()
}

pub(in crate::graphics::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_visibility_plan_trace_regions(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiVisibilityPlanTraceRegion> {
    hybrid_gi_extract_trace_region_records(extract)
        .into_iter()
        .map(HybridGiVisibilityPlanTraceRegion::from)
        .collect()
}

impl From<HybridGiExtractProbeRecord> for HybridGiVisibilityPlanProbe {
    fn from(probe: HybridGiExtractProbeRecord) -> Self {
        Self {
            entity: probe.entity,
            probe_id: probe.probe_id,
            position: probe.position,
            radius: probe.radius,
            parent_probe_id: probe.parent_probe_id,
            resident: probe.resident,
            ray_budget: probe.ray_budget,
        }
    }
}

impl From<HybridGiExtractTraceRegionRecord> for HybridGiVisibilityPlanTraceRegion {
    fn from(region: HybridGiExtractTraceRegionRecord) -> Self {
        Self {
            entity: region.entity,
            region_id: region.region_id,
            bounds_center: region.bounds_center,
            bounds_radius: region.bounds_radius,
            screen_coverage: region.screen_coverage,
        }
    }
}
