use std::collections::BTreeSet;

use crate::hybrid_gi::types::{
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};
use zircon_runtime::core::math::Vec3;

use super::trace_region_limits::MAX_GPU_TRACE_REGION_INPUTS;

pub(super) const NO_PARENT_PROBE_ID: u32 = u32::MAX;
const RESIDENT_ANCESTOR_SLOTS: usize = 4;
const ANCESTOR_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const MIN_TRACE_SUPPORT_REACH: f32 = 0.0001;
const LINEAGE_TRACE_SUPPORT_MAX_SCORE: f32 = 4.0;
const PROBE_POSITION_SCALE: f32 = 64.0;
const PROBE_POSITION_BIAS: i32 = 2048;
const PROBE_RADIUS_SCALE: f32 = 96.0;
const TRACE_COVERAGE_SCALE: f32 = 128.0;

pub(super) fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16)
}

pub(super) fn probe_position_x_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, probe_id)
        .map(|scene_data| scene_data.position_x_q())
        .unwrap_or_default()
}

pub(super) fn probe_position_y_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, probe_id)
        .map(|scene_data| scene_data.position_y_q())
        .unwrap_or_default()
}

pub(super) fn probe_position_z_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, probe_id)
        .map(|scene_data| scene_data.position_z_q())
        .unwrap_or_default()
}

pub(super) fn probe_radius_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, probe_id)
        .map(|scene_data| scene_data.radius_q())
        .unwrap_or_default()
}

pub(super) fn probe_parent_probe_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> u32 {
    parent_probe_id(resolve_runtime, probe_id).unwrap_or(NO_PARENT_PROBE_ID)
}

pub(super) fn probe_resident_ancestors(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
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
        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, current_probe_id) else {
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
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let support_score =
        probe_lineage_trace_support_score(resolve_runtime, scheduled_trace_region_ids, probe_id);
    ((support_score / LINEAGE_TRACE_SUPPORT_MAX_SCORE).clamp(0.0, 1.0) * 255.0).round() as u32
}

pub(super) fn probe_lineage_trace_lighting_rgb(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(resolve_runtime, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe_scene_data) = probe_scene_data(resolve_runtime, current_probe_id) else {
            break;
        };
        for region in &scheduled_trace_regions {
            let region_rt_lighting_rgb = region.rt_lighting_rgb();
            if region_rt_lighting_rgb == [0, 0, 0] {
                continue;
            }
            let support =
                single_probe_trace_support_score(probe_scene_data, region) * lineage_weight;
            if support <= f32::EPSILON {
                continue;
            }
            weighted_rgb[0] += region_rt_lighting_rgb[0] as f32 / 255.0 * support;
            weighted_rgb[1] += region_rt_lighting_rgb[1] as f32 / 255.0 * support;
            weighted_rgb[2] += region_rt_lighting_rgb[2] as f32 / 255.0 * support;
            total_support += support;
        }

        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, current_probe_id) else {
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

pub(super) fn scheduled_live_trace_region_ids(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<u32> {
    resolve_runtime
        .map(|runtime| scheduled_runtime_trace_region_ids(runtime, scheduled_trace_region_ids))
        .unwrap_or_default()
}

fn scheduled_runtime_trace_region_ids(
    resolve_runtime: &HybridGiResolveRuntime,
    scheduled_trace_region_ids: &[u32],
) -> Vec<u32> {
    let mut scheduled_region_ids = BTreeSet::new();
    scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .filter(|region_id| {
            resolve_runtime
                .trace_region_scene_data(*region_id)
                .is_some()
        })
        .take(MAX_GPU_TRACE_REGION_INPUTS)
        .collect()
}

fn probe_lineage_trace_support_score(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> f32 {
    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(resolve_runtime, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0.0;
    }

    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe_scene_data) = probe_scene_data(resolve_runtime, current_probe_id) else {
            break;
        };
        total_support += scheduled_trace_regions
            .iter()
            .map(|region| single_probe_trace_support_score(probe_scene_data, region))
            .sum::<f32>()
            * lineage_weight;

        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, current_probe_id) else {
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
    probe_scene_data: HybridGiResolveProbeSceneData,
    region: &HybridGiResolveTraceRegionSceneData,
) -> f32 {
    let probe_position = dequantized_probe_position(probe_scene_data);
    let probe_radius = dequantized_probe_radius(probe_scene_data);
    let trace_region_center = dequantized_trace_region_center(*region);
    let trace_region_radius = dequantized_trace_region_radius(*region);
    let reach = (trace_region_radius + probe_radius).max(MIN_TRACE_SUPPORT_REACH);
    let distance_to_region = probe_position.distance(trace_region_center);
    let falloff = (1.0 - distance_to_region / reach).max(0.0);
    falloff * falloff * dequantized_trace_region_coverage(*region).max(0.0)
}

fn dequantized_probe_position(probe_scene_data: HybridGiResolveProbeSceneData) -> Vec3 {
    Vec3::new(
        dequantized_signed(probe_scene_data.position_x_q()),
        dequantized_signed(probe_scene_data.position_y_q()),
        dequantized_signed(probe_scene_data.position_z_q()),
    )
}

fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - PROBE_POSITION_BIAS) as f32 / PROBE_POSITION_SCALE
}

fn dequantized_probe_radius(probe_scene_data: HybridGiResolveProbeSceneData) -> f32 {
    probe_scene_data.radius_q() as f32 / PROBE_RADIUS_SCALE
}

fn dequantized_trace_region_center(region: HybridGiResolveTraceRegionSceneData) -> Vec3 {
    Vec3::new(
        dequantized_signed(region.center_x_q()),
        dequantized_signed(region.center_y_q()),
        dequantized_signed(region.center_z_q()),
    )
}

fn dequantized_trace_region_radius(region: HybridGiResolveTraceRegionSceneData) -> f32 {
    region.radius_q() as f32 / PROBE_RADIUS_SCALE
}

fn dequantized_trace_region_coverage(region: HybridGiResolveTraceRegionSceneData) -> f32 {
    region.coverage_q() as f32 / TRACE_COVERAGE_SCALE
}

pub(super) fn scheduled_trace_regions_by_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<HybridGiResolveTraceRegionSceneData> {
    scheduled_trace_region_scene_data_by_id(resolve_runtime, scheduled_trace_region_ids)
        .into_iter()
        .map(|(_, scene_data)| scene_data)
        .collect()
}

pub(super) fn scheduled_trace_region_scene_data_by_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<(u32, HybridGiResolveTraceRegionSceneData)> {
    let Some(runtime) = resolve_runtime else {
        return Vec::new();
    };
    scheduled_runtime_trace_region_ids(runtime, scheduled_trace_region_ids)
        .into_iter()
        .filter_map(|region_id| {
            runtime
                .trace_region_scene_data(region_id)
                .map(|scene_data| (region_id, scene_data))
        })
        .collect()
}

fn probe_scene_data(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> Option<HybridGiResolveProbeSceneData> {
    resolve_runtime.and_then(|runtime| runtime.probe_scene_data(probe_id))
}

fn parent_probe_id(resolve_runtime: Option<&HybridGiResolveRuntime>, probe_id: u32) -> Option<u32> {
    resolve_runtime.and_then(|runtime| runtime.parent_probe_id(probe_id))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;

    #[test]
    fn probe_quantization_and_lineage_use_resolve_runtime_scene_truth() {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([
                (3, HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 192)),
                (7, HybridGiResolveProbeSceneData::new(2112, 2048, 2048, 96)),
            ]))
            .with_probe_parent_probes(BTreeMap::from([(7, 3)]))
            .with_trace_region_scene_data(BTreeMap::from([(
                9,
                HybridGiResolveTraceRegionSceneData::new(2112, 2048, 2048, 192, 128, [64, 96, 128]),
            )]))
            .build();

        assert_eq!(probe_position_x_q(Some(&runtime), 7), 2112);
        assert_eq!(probe_radius_q(Some(&runtime), 7), 96);
        assert_eq!(probe_parent_probe_id(Some(&runtime), 7), 3);
        assert_eq!(
            probe_resident_ancestors(Some(&runtime), &BTreeSet::from([3]), 7)[0],
            (3, 1)
        );
        assert!(probe_lineage_trace_support_q(Some(&runtime), &[9], 7) > 0);
        assert_eq!(
            probe_lineage_trace_lighting_rgb(Some(&runtime), &[9], 7),
            pack_rgb8([64, 96, 128])
        );
    }

    #[test]
    fn scheduled_trace_regions_are_deduplicated_and_require_runtime_scene_data() {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(BTreeMap::from([(
                9,
                HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, [1, 2, 3]),
            )]))
            .build();

        assert_eq!(
            scheduled_live_trace_region_ids(Some(&runtime), &[9, 9, 404]),
            vec![9]
        );
        assert!(scheduled_live_trace_region_ids(None, &[9]).is_empty());
    }
}
