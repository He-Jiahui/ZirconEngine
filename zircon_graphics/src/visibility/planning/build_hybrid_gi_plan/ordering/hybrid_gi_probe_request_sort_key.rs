use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use zircon_scene::{RenderHybridGiProbe, RenderHybridGiTraceRegion};

const ANCESTOR_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const MIN_TRACE_SUPPORT_REACH: f32 = 0.0001;

pub(in crate::visibility::planning::build_hybrid_gi_plan) fn hybrid_gi_probe_request_sort_key(
    left: &RenderHybridGiProbe,
    right: &RenderHybridGiProbe,
    scheduled_trace_regions: &[RenderHybridGiTraceRegion],
    probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
) -> Ordering {
    probe_trace_support_score(right, scheduled_trace_regions, probes_by_id)
        .total_cmp(&probe_trace_support_score(
            left,
            scheduled_trace_regions,
            probes_by_id,
        ))
        .then_with(|| {
            probe_hierarchy_depth(right, probes_by_id)
                .cmp(&probe_hierarchy_depth(left, probes_by_id))
        })
        .then_with(|| right.ray_budget.cmp(&left.ray_budget))
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}

fn probe_hierarchy_depth(
    probe: &RenderHybridGiProbe,
    probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
) -> usize {
    let mut depth = 0usize;
    let mut current_probe_id = probe.probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe.probe_id]);

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        depth += 1;
        current_probe_id = parent_probe_id;
    }

    depth
}

fn probe_trace_support_score(
    probe: &RenderHybridGiProbe,
    scheduled_trace_regions: &[RenderHybridGiTraceRegion],
    probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
) -> f32 {
    let mut total_support = single_probe_trace_support_score(probe, scheduled_trace_regions);
    let mut current_probe_id = probe.probe_id;
    let mut lineage_weight = ANCESTOR_TRACE_SUPPORT_FALLOFF;
    let mut visited_probe_ids = BTreeSet::from([probe.probe_id]);

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        let Some(parent_probe) = probes_by_id.get(&parent_probe_id) else {
            break;
        };

        total_support += single_probe_trace_support_score(parent_probe, scheduled_trace_regions)
            * lineage_weight;
        lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
        current_probe_id = parent_probe_id;
    }

    total_support
}

fn single_probe_trace_support_score(
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
