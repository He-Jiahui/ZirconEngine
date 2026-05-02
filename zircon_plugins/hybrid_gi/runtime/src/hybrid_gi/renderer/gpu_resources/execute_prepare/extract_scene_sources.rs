use std::collections::{BTreeMap, BTreeSet};

use crate::hybrid_gi::types::{HybridGiResolveProbeSceneData, HybridGiResolveTraceRegionSceneData};
use zircon_runtime::core::framework::render::RenderHybridGiExtract;
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_probe_records_by_id,
    hybrid_gi_extract_trace_region_records_by_id,
    hybrid_gi_extract_uses_scene_representation_budget,
};

use super::super::seed_quantization::{quantized_positive, quantized_signed};

const PROBE_RADIUS_SCALE: f32 = 96.0;
const TRACE_COVERAGE_SCALE: f32 = 128.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HybridGiPrepareProbeSceneSource {
    pub(super) scene_data: HybridGiResolveProbeSceneData,
    pub(super) parent_probe_id: Option<u32>,
}

pub(super) fn extract_trace_region_ids(extract: Option<&RenderHybridGiExtract>) -> BTreeSet<u32> {
    let Some(extract) = enabled_hybrid_gi_extract(extract) else {
        return BTreeSet::new();
    };

    hybrid_gi_extract_trace_region_records_by_id(extract)
        .into_keys()
        .collect()
}

pub(super) fn fallback_trace_region_scene_data_by_id(
    extract: Option<&RenderHybridGiExtract>,
) -> BTreeMap<u32, HybridGiResolveTraceRegionSceneData> {
    let Some(extract) = fallback_enabled_extract(extract) else {
        return BTreeMap::new();
    };

    hybrid_gi_extract_trace_region_records_by_id(extract)
        .into_iter()
        .map(|(region_id, region)| {
            (
                region_id,
                HybridGiResolveTraceRegionSceneData::new(
                    quantized_signed(region.bounds_center.x),
                    quantized_signed(region.bounds_center.y),
                    quantized_signed(region.bounds_center.z),
                    quantized_positive(region.bounds_radius, PROBE_RADIUS_SCALE),
                    quantized_positive(region.screen_coverage, TRACE_COVERAGE_SCALE),
                    region.rt_lighting_rgb,
                ),
            )
        })
        .collect()
}

pub(super) fn fallback_probe_scene_sources_by_id(
    extract: Option<&RenderHybridGiExtract>,
) -> BTreeMap<u32, HybridGiPrepareProbeSceneSource> {
    let Some(extract) = fallback_enabled_extract(extract) else {
        return BTreeMap::new();
    };

    hybrid_gi_extract_probe_records_by_id(extract)
        .into_iter()
        .map(|(probe_id, probe)| {
            (
                probe_id,
                HybridGiPrepareProbeSceneSource {
                    scene_data: HybridGiResolveProbeSceneData::new(
                        quantized_signed(probe.position.x),
                        quantized_signed(probe.position.y),
                        quantized_signed(probe.position.z),
                        quantized_positive(probe.radius, PROBE_RADIUS_SCALE),
                    ),
                    parent_probe_id: probe.parent_probe_id,
                },
            )
        })
        .collect()
}

fn fallback_enabled_extract(
    extract: Option<&RenderHybridGiExtract>,
) -> Option<&RenderHybridGiExtract> {
    enabled_hybrid_gi_extract(extract)
        .filter(|extract| !hybrid_gi_extract_uses_scene_representation_budget(extract))
}
