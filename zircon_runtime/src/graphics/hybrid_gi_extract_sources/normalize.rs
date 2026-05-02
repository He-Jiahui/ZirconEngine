use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
};

use super::{HybridGiExtractProbeRecord, HybridGiExtractTraceRegionRecord};

pub fn hybrid_gi_extract_probe_records(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiExtractProbeRecord> {
    first_hybrid_gi_probe_records(&extract.probes)
}

pub fn hybrid_gi_extract_probe_records_by_id(
    extract: &RenderHybridGiExtract,
) -> BTreeMap<u32, HybridGiExtractProbeRecord> {
    hybrid_gi_extract_probe_records(extract)
        .into_iter()
        .map(|probe| (probe.probe_id, probe))
        .collect()
}

pub fn hybrid_gi_extract_trace_region_records(
    extract: &RenderHybridGiExtract,
) -> Vec<HybridGiExtractTraceRegionRecord> {
    first_hybrid_gi_trace_region_records(&extract.trace_regions)
}

pub fn hybrid_gi_extract_trace_region_records_by_id(
    extract: &RenderHybridGiExtract,
) -> BTreeMap<u32, HybridGiExtractTraceRegionRecord> {
    hybrid_gi_extract_trace_region_records(extract)
        .into_iter()
        .map(|region| (region.region_id, region))
        .collect()
}

fn first_hybrid_gi_probe_records(
    probes: &[RenderHybridGiProbe],
) -> Vec<HybridGiExtractProbeRecord> {
    let mut registered_probe_ids = BTreeSet::new();
    let live_probe_payloads = probes
        .iter()
        .copied()
        .filter(|probe| registered_probe_ids.insert(probe.probe_id))
        .collect::<Vec<_>>();
    let live_probe_ids = live_probe_payloads
        .iter()
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    let mut probe_parent_probes = BTreeMap::new();

    for probe in &live_probe_payloads {
        if let Some(parent_probe_id) = probe
            .parent_probe_id
            .filter(|parent_probe_id| live_probe_ids.contains(parent_probe_id))
            .filter(|parent_probe_id| {
                !parent_link_would_cycle(&probe_parent_probes, probe.probe_id, *parent_probe_id)
            })
        {
            probe_parent_probes.insert(probe.probe_id, parent_probe_id);
        }
    }

    live_probe_payloads
        .into_iter()
        .map(|mut probe| {
            probe.parent_probe_id = probe_parent_probes.get(&probe.probe_id).copied();
            probe_record_from_render(probe)
        })
        .collect()
}

fn first_hybrid_gi_trace_region_records(
    trace_regions: &[RenderHybridGiTraceRegion],
) -> Vec<HybridGiExtractTraceRegionRecord> {
    let mut registered_trace_region_ids = BTreeSet::new();
    trace_regions
        .iter()
        .copied()
        .filter(|region| registered_trace_region_ids.insert(region.region_id))
        .map(trace_region_record_from_render)
        .collect()
}

fn probe_record_from_render(probe: RenderHybridGiProbe) -> HybridGiExtractProbeRecord {
    HybridGiExtractProbeRecord {
        entity: probe.entity,
        probe_id: probe.probe_id,
        position: probe.position,
        radius: probe.radius,
        parent_probe_id: probe.parent_probe_id,
        resident: probe.resident,
        ray_budget: probe.ray_budget,
    }
}

fn trace_region_record_from_render(
    region: RenderHybridGiTraceRegion,
) -> HybridGiExtractTraceRegionRecord {
    HybridGiExtractTraceRegionRecord {
        entity: region.entity,
        region_id: region.region_id,
        bounds_center: region.bounds_center,
        bounds_radius: region.bounds_radius,
        screen_coverage: region.screen_coverage,
        rt_lighting_rgb: region.rt_lighting_rgb,
    }
}

fn parent_link_would_cycle(
    probe_parent_probes: &BTreeMap<u32, u32>,
    probe_id: u32,
    parent_probe_id: u32,
) -> bool {
    let mut current_probe_id = parent_probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        if !visited_probe_ids.insert(current_probe_id) {
            return true;
        }
        let Some(next_parent_probe_id) = probe_parent_probes.get(&current_probe_id).copied() else {
            return false;
        };
        current_probe_id = next_parent_probe_id;
    }
}
