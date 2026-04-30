use crate::core::framework::render::RenderHybridGiExtract;
use crate::core::math::{Real, Vec3};
use crate::graphics::hybrid_gi_extract_sources::{
    hybrid_gi_extract_probe_records, hybrid_gi_extract_trace_region_records,
    HybridGiExtractProbeRecord, HybridGiExtractTraceRegionRecord,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::runtime::hybrid_gi) struct HybridGiExtractProbePayload {
    pub(in crate::graphics::runtime::hybrid_gi) probe_id: u32,
    pub(in crate::graphics::runtime::hybrid_gi) position: Vec3,
    pub(in crate::graphics::runtime::hybrid_gi) radius: Real,
    pub(in crate::graphics::runtime::hybrid_gi) parent_probe_id: Option<u32>,
    pub(in crate::graphics::runtime::hybrid_gi) resident: bool,
    pub(in crate::graphics::runtime::hybrid_gi) ray_budget: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::runtime::hybrid_gi) struct HybridGiExtractTraceRegionPayload {
    pub(in crate::graphics::runtime::hybrid_gi) region_id: u32,
    pub(in crate::graphics::runtime::hybrid_gi) bounds_center: Vec3,
    pub(in crate::graphics::runtime::hybrid_gi) bounds_radius: Real,
    pub(in crate::graphics::runtime::hybrid_gi) screen_coverage: Real,
    pub(in crate::graphics::runtime::hybrid_gi) rt_lighting_rgb: [u8; 3],
}

pub(in crate::graphics::runtime::hybrid_gi) fn first_hybrid_gi_runtime_probe_payloads(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiExtractProbePayload> {
    hybrid_gi_extract_probe_records(extract)
        .into_iter()
        .map(HybridGiExtractProbePayload::from)
        .collect()
}

pub(in crate::graphics::runtime::hybrid_gi) fn first_hybrid_gi_runtime_trace_region_payloads(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiExtractTraceRegionPayload> {
    hybrid_gi_extract_trace_region_records(extract)
        .into_iter()
        .map(HybridGiExtractTraceRegionPayload::from)
        .collect()
}

impl From<HybridGiExtractProbeRecord> for HybridGiExtractProbePayload {
    fn from(probe: HybridGiExtractProbeRecord) -> Self {
        Self {
            probe_id: probe.probe_id,
            position: probe.position,
            radius: probe.radius,
            parent_probe_id: probe.parent_probe_id,
            resident: probe.resident,
            ray_budget: probe.ray_budget,
        }
    }
}

impl From<HybridGiExtractTraceRegionRecord> for HybridGiExtractTraceRegionPayload {
    fn from(region: HybridGiExtractTraceRegionRecord) -> Self {
        Self {
            region_id: region.region_id,
            bounds_center: region.bounds_center,
            bounds_radius: region.bounds_radius,
            screen_coverage: region.screen_coverage,
            rt_lighting_rgb: region.rt_lighting_rgb,
        }
    }
}
