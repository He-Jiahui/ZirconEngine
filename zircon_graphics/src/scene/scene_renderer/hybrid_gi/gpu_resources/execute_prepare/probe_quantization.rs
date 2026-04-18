use std::collections::BTreeSet;

use zircon_scene::{RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion};

use super::super::seed_quantization::{quantized_positive, quantized_signed};

pub(super) const NO_PARENT_PROBE_ID: u32 = u32::MAX;
const RESIDENT_ANCESTOR_SLOTS: usize = 4;
const ANCESTOR_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const MIN_TRACE_SUPPORT_REACH: f32 = 0.0001;
const LINEAGE_TRACE_SUPPORT_MAX_SCORE: f32 = 4.0;

pub(super) fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16)
}

pub(super) fn probe_position_x_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.x))
        .unwrap_or_default()
}

pub(super) fn probe_position_y_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.y))
        .unwrap_or_default()
}

pub(super) fn probe_position_z_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.z))
        .unwrap_or_default()
}

pub(super) fn probe_radius_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_positive(probe.radius, 96.0))
        .unwrap_or_default()
}

pub(super) fn probe_parent_probe_id(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .and_then(|probe| probe.parent_probe_id)
        .unwrap_or(NO_PARENT_PROBE_ID)
}

pub(super) fn probe_resident_ancestors(
    extract: Option<&RenderHybridGiExtract>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> [(u32, u32); RESIDENT_ANCESTOR_SLOTS] {
    if resident_probe_ids.is_empty() {
        return [(NO_PARENT_PROBE_ID, 0); RESIDENT_ANCESTOR_SLOTS];
    }

    let mut current_probe_id = probe_id;
    let mut resident_ancestor_depth = 0_u32;
    let mut resident_ancestors = [(NO_PARENT_PROBE_ID, 0); RESIDENT_ANCESTOR_SLOTS];
    let mut resident_ancestor_count = 0usize;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(parent_probe_id) =
            probe_from_extract(extract, current_probe_id).and_then(|probe| probe.parent_probe_id)
        else {
            return resident_ancestors;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            return resident_ancestors;
        }

        resident_ancestor_depth = resident_ancestor_depth.saturating_add(1);
        if resident_probe_ids.contains(&parent_probe_id) {
            resident_ancestors[resident_ancestor_count] =
                (parent_probe_id, resident_ancestor_depth);
            resident_ancestor_count += 1;
            if resident_ancestor_count == resident_ancestors.len() {
                return resident_ancestors;
            }
        }
        current_probe_id = parent_probe_id;
    }
}

pub(super) fn probe_lineage_trace_support_q(
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let support_score =
        probe_lineage_trace_support_score(extract, scheduled_trace_region_ids, probe_id);
    ((support_score / LINEAGE_TRACE_SUPPORT_MAX_SCORE).clamp(0.0, 1.0) * 255.0).round() as u32
}

pub(super) fn probe_lineage_trace_lighting_rgb(
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let Some(extract) = extract else {
        return 0;
    };

    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(extract, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe) = probe_from_extract(Some(extract), current_probe_id) else {
            break;
        };
        for region in &scheduled_trace_regions {
            if region.rt_lighting_rgb == [0, 0, 0] {
                continue;
            }
            let support = single_probe_trace_support_score(probe, region) * lineage_weight;
            if support <= f32::EPSILON {
                continue;
            }
            weighted_rgb[0] += region.rt_lighting_rgb[0] as f32 / 255.0 * support;
            weighted_rgb[1] += region.rt_lighting_rgb[1] as f32 / 255.0 * support;
            weighted_rgb[2] += region.rt_lighting_rgb[2] as f32 / 255.0 * support;
            total_support += support;
        }

        let Some(parent_probe_id) = probe.parent_probe_id else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
        current_probe_id = parent_probe_id;
    }

    if total_support <= f32::EPSILON {
        return 0;
    }

    pack_rgb8([
        ((weighted_rgb[0] / total_support).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((weighted_rgb[1] / total_support).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((weighted_rgb[2] / total_support).clamp(0.0, 1.0) * 255.0).round() as u8,
    ])
}

fn probe_lineage_trace_support_score(
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> f32 {
    let Some(extract) = extract else {
        return 0.0;
    };

    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(extract, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0.0;
    }

    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe) = probe_from_extract(Some(extract), current_probe_id) else {
            break;
        };
        total_support += scheduled_trace_regions
            .iter()
            .map(|region| single_probe_trace_support_score(probe, region))
            .sum::<f32>()
            * lineage_weight;

        let Some(parent_probe_id) = probe.parent_probe_id else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
        current_probe_id = parent_probe_id;
    }

    total_support
}

fn single_probe_trace_support_score(
    probe: &RenderHybridGiProbe,
    region: &RenderHybridGiTraceRegion,
) -> f32 {
    let reach = (region.bounds_radius + probe.radius).max(MIN_TRACE_SUPPORT_REACH);
    let distance_to_region = probe.position.distance(region.bounds_center);
    let falloff = (1.0 - distance_to_region / reach).max(0.0);
    falloff * falloff * region.screen_coverage.max(0.0)
}

fn scheduled_trace_regions_by_id<'a>(
    extract: &'a RenderHybridGiExtract,
    scheduled_trace_region_ids: &[u32],
) -> Vec<&'a RenderHybridGiTraceRegion> {
    scheduled_trace_region_ids
        .iter()
        .filter_map(|region_id| {
            extract
                .trace_regions
                .iter()
                .find(|region| region.region_id == *region_id)
        })
        .collect()
}

fn probe_from_extract(
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<&RenderHybridGiProbe> {
    extract.and_then(|extract| {
        extract
            .probes
            .iter()
            .find(|probe| probe.probe_id == probe_id)
    })
}
