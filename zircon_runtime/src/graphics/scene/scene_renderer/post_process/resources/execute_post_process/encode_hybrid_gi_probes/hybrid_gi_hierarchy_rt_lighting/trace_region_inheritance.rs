use std::collections::{BTreeMap, BTreeSet};

use crate::graphics::types::{HybridGiPrepareProbe, ViewportRenderFrame};

use super::super::super::hybrid_gi_trace_region_source::HybridGiTraceRegionSource;
use super::super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::super::runtime_parent_chain::runtime_parent_topology_is_authoritative;
use super::runtime_rt_sources::{
    hierarchy_trace_region_support, hybrid_gi_trace_region_rt_lighting,
};

const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;

pub(super) fn trace_region_inheritance_rt_lighting<
    S: HybridGiProbeSource + ?Sized,
    P: HybridGiProbeSource,
    R: HybridGiTraceRegionSource,
>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_prepare_voxel_fallback: Option<[f32; 4]>,
    scheduled_trace_region_ids: &[u32],
    probes_by_id: &BTreeMap<u32, P>,
    trace_regions_by_id: &BTreeMap<u32, R>,
    resident_prepare_by_id: &BTreeMap<u32, &HybridGiPrepareProbe>,
) -> [f32; 4] {
    if scheduled_trace_region_ids.is_empty() {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }
    if runtime_parent_topology_is_authoritative(frame) {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut current_probe_id = source.probe_id();
    let mut visited_probe_ids = BTreeSet::from([source.probe_id()]);
    let mut ancestor_depth = 0usize;

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id())
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        let Some(ancestor_probe) = probes_by_id.get(&parent_probe_id) else {
            break;
        };
        let resident_budget_weight = resident_prepare_by_id
            .get(&parent_probe_id)
            .map(|probe| hybrid_gi_budget_weight(probe.ray_budget))
            .unwrap_or(0.0);
        if resident_budget_weight <= f32::EPSILON {
            current_probe_id = parent_probe_id;
            continue;
        }

        ancestor_depth += 1;
        let hierarchy_weight =
            ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi((ancestor_depth.saturating_sub(1)) as i32);
        for region in scheduled_trace_region_ids
            .iter()
            .filter_map(|region_id| trace_regions_by_id.get(region_id))
        {
            let region_rt_lighting = hybrid_gi_trace_region_rt_lighting(region);
            if region_rt_lighting[3] <= 0.0 {
                continue;
            }

            let support = hierarchy_weight
                * resident_budget_weight
                * region_rt_lighting[3]
                * hierarchy_trace_region_support(ancestor_probe, region);
            if support <= 0.0 {
                continue;
            }

            weighted_rgb[0] += region_rt_lighting[0] * support;
            weighted_rgb[1] += region_rt_lighting[1] * support;
            weighted_rgb[2] += region_rt_lighting[2] * support;
            total_support += support;
        }

        current_probe_id = parent_probe_id;
    }

    if total_support <= f32::EPSILON {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    let inherited_weight = (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75);
    [
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        inherited_weight,
    ]
}
