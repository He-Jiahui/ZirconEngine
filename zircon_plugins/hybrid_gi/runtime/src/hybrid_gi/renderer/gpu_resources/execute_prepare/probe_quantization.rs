use std::collections::BTreeSet;

use crate::hybrid_gi::types::{
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};
use zircon_runtime::core::framework::render::RenderHybridGiExtract;
use zircon_runtime::core::math::Vec3;

use super::extract_scene_sources::{
    extract_trace_region_ids, fallback_probe_scene_sources_by_id,
    fallback_trace_region_scene_data_by_id,
};
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
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, extract, probe_id)
        .map(|scene_data| scene_data.position_x_q())
        .unwrap_or_default()
}

pub(super) fn probe_position_y_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, extract, probe_id)
        .map(|scene_data| scene_data.position_y_q())
        .unwrap_or_default()
}

pub(super) fn probe_position_z_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, extract, probe_id)
        .map(|scene_data| scene_data.position_z_q())
        .unwrap_or_default()
}

pub(super) fn probe_radius_q(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> u32 {
    probe_scene_data(resolve_runtime, extract, probe_id)
        .map(|scene_data| scene_data.radius_q())
        .unwrap_or_default()
}

pub(super) fn probe_parent_probe_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> u32 {
    parent_probe_id(resolve_runtime, extract, probe_id).unwrap_or(NO_PARENT_PROBE_ID)
}

pub(super) fn probe_resident_ancestors(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
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
        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, extract, current_probe_id)
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
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let support_score = probe_lineage_trace_support_score(
        resolve_runtime,
        extract,
        scheduled_trace_region_ids,
        probe_id,
    );
    ((support_score / LINEAGE_TRACE_SUPPORT_MAX_SCORE).clamp(0.0, 1.0) * 255.0).round() as u32
}

pub(super) fn probe_lineage_trace_lighting_rgb(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> u32 {
    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(resolve_runtime, extract, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe_scene_data) = probe_scene_data(resolve_runtime, extract, current_probe_id)
        else {
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

        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, extract, current_probe_id)
        else {
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
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<u32> {
    if let Some(runtime) = resolve_runtime {
        return scheduled_runtime_trace_region_ids(runtime, extract, scheduled_trace_region_ids);
    }

    let trace_regions_by_id = fallback_trace_region_scene_data_by_id(extract);
    let mut scheduled_region_ids = BTreeSet::new();
    scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .filter(|region_id| trace_regions_by_id.contains_key(region_id))
        .take(MAX_GPU_TRACE_REGION_INPUTS)
        .collect()
}

fn scheduled_runtime_trace_region_ids(
    resolve_runtime: &HybridGiResolveRuntime,
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<u32> {
    let extract_backed_trace_region_ids = if runtime_has_scene_truth(resolve_runtime) {
        extract_trace_region_ids(extract)
    } else {
        BTreeSet::new()
    };
    let mut scheduled_region_ids = BTreeSet::new();
    scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .filter(|region_id| !extract_backed_trace_region_ids.contains(region_id))
        .filter(|region_id| {
            resolve_runtime
                .trace_region_scene_data(*region_id)
                .is_some()
        })
        .take(MAX_GPU_TRACE_REGION_INPUTS)
        .collect()
}

fn runtime_has_scene_truth(runtime: &HybridGiResolveRuntime) -> bool {
    runtime
        .scene_truth_irradiance_probe_ids()
        .any(|probe_id| runtime_probe_has_irradiance_scene_truth(runtime, probe_id))
        || runtime
            .scene_truth_rt_lighting_probe_ids()
            .any(|probe_id| runtime_probe_has_rt_lighting_scene_truth(runtime, probe_id))
}

fn runtime_probe_has_irradiance_scene_truth(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> bool {
    runtime.hierarchy_irradiance_includes_scene_truth(probe_id)
        && runtime
            .hierarchy_irradiance(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false)
}

fn runtime_probe_has_rt_lighting_scene_truth(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> bool {
    runtime.hierarchy_rt_lighting_includes_scene_truth(probe_id)
        && (runtime
            .hierarchy_rt_lighting(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false)
            || runtime.has_probe_rt_lighting(probe_id))
}

fn probe_lineage_trace_support_score(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
    probe_id: u32,
) -> f32 {
    let scheduled_trace_regions =
        scheduled_trace_regions_by_id(resolve_runtime, extract, scheduled_trace_region_ids);
    if scheduled_trace_regions.is_empty() {
        return 0.0;
    }

    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(probe_scene_data) = probe_scene_data(resolve_runtime, extract, current_probe_id)
        else {
            break;
        };
        total_support += scheduled_trace_regions
            .iter()
            .map(|region| single_probe_trace_support_score(probe_scene_data, region))
            .sum::<f32>()
            * lineage_weight;

        let Some(parent_probe_id) = parent_probe_id(resolve_runtime, extract, current_probe_id)
        else {
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
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<HybridGiResolveTraceRegionSceneData> {
    scheduled_trace_region_scene_data_by_id(resolve_runtime, extract, scheduled_trace_region_ids)
        .into_iter()
        .map(|(_, scene_data)| scene_data)
        .collect()
}

pub(super) fn scheduled_trace_region_scene_data_by_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    scheduled_trace_region_ids: &[u32],
) -> Vec<(u32, HybridGiResolveTraceRegionSceneData)> {
    if let Some(runtime) = resolve_runtime {
        return scheduled_runtime_trace_region_ids(runtime, extract, scheduled_trace_region_ids)
            .into_iter()
            .filter_map(|region_id| {
                runtime
                    .trace_region_scene_data(region_id)
                    .map(|scene_data| (region_id, scene_data))
            })
            .collect();
    }

    let trace_regions_by_id = fallback_trace_region_scene_data_by_id(extract);
    let mut scheduled_region_ids = BTreeSet::new();
    scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .into_iter()
        .filter_map(|region_id| {
            trace_regions_by_id
                .get(&region_id)
                .copied()
                .map(|scene_data| (region_id, scene_data))
        })
        .take(MAX_GPU_TRACE_REGION_INPUTS)
        .collect()
}

fn probe_scene_data_from_extract(
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<HybridGiResolveProbeSceneData> {
    fallback_probe_scene_sources_by_id(extract)
        .get(&probe_id)
        .map(|source| source.scene_data)
}

fn probe_scene_data(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<HybridGiResolveProbeSceneData> {
    resolve_runtime
        .and_then(|runtime| runtime.probe_scene_data(probe_id))
        .or_else(|| probe_scene_data_from_extract(extract, probe_id))
}

fn parent_probe_id(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<u32> {
    if let Some(runtime) = resolve_runtime {
        return runtime.parent_probe_id(probe_id);
    }
    parent_probe_id_from_extract(extract, probe_id)
}

fn parent_probe_id_from_extract(
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<u32> {
    fallback_probe_scene_sources_by_id(extract)
        .get(&probe_id)
        .and_then(|source| source.parent_probe_id)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::hybrid_gi::types::{
        HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
    };
    use zircon_runtime::core::framework::render::{
        RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    };
    use zircon_runtime::core::math::Vec3;

    use super::*;

    #[test]
    fn probe_parent_probe_id_prefers_runtime_parent_topology_over_legacy_extract() {
        let extract = extract_with_probes(vec![
            probe(100, Vec3::ZERO, 1.0),
            probe(200, Vec3::ZERO, 1.0),
            probe_with_parent(300, 100, Vec3::ZERO, 1.0),
        ]);
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_parent_probes(BTreeMap::from([(300, 200)]))
            .build();

        assert_eq!(
            probe_parent_probe_id(Some(&runtime), Some(&extract), 300),
            200
        );
    }

    #[test]
    fn probe_lineage_trace_support_treats_flat_runtime_parent_topology_as_authoritative() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            tracing_budget: 1,
            probes: vec![
                probe(100, Vec3::ZERO, 1.0),
                probe_with_parent(300, 100, Vec3::new(10.0, 0.0, 0.0), 0.1),
            ],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 1.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(BTreeMap::from([(
                40,
                HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, [255, 128, 64]),
            )]))
            .build();

        assert!(
            probe_lineage_trace_support_q(None, Some(&extract), &[40], 300) > 0,
            "legacy extract parent topology should reach the warm parent without runtime authority"
        );
        assert_eq!(
            probe_lineage_trace_support_q(Some(&runtime), Some(&extract), &[40], 300),
            0,
            "flat runtime topology should block stale legacy RenderHybridGiProbe parent inheritance"
        );
    }

    #[test]
    fn probe_resident_ancestors_drop_legacy_parent_without_live_payload() {
        let extract = extract_with_probes(vec![probe_with_parent(300, 400, Vec3::ZERO, 1.0)]);

        assert_eq!(
            probe_resident_ancestors(None, Some(&extract), &BTreeSet::from([400]), 300)[0],
            (NO_PARENT_PROBE_ID, 0),
            "legacy parent ids without a live RenderHybridGiProbe payload should not re-enter GPU prepare ancestry"
        );
    }

    #[test]
    fn probe_lineage_trace_support_deduplicates_scheduled_live_payload_ids() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            probes: vec![probe(100, Vec3::ZERO, 0.5)],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 0.2,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert_eq!(
            probe_lineage_trace_support_q(None, Some(&extract), &[40], 100),
            probe_lineage_trace_support_q(None, Some(&extract), &[40, 40], 100)
        );
    }

    #[test]
    fn probe_lineage_trace_support_limits_legacy_schedule_before_tail_payload() {
        let tail_region_id = 10_000;
        let mut scheduled_trace_region_ids =
            (0..MAX_GPU_TRACE_REGION_INPUTS as u32).collect::<Vec<_>>();
        scheduled_trace_region_ids.push(tail_region_id);
        let extract = RenderHybridGiExtract {
            enabled: true,
            probes: vec![probe(100, Vec3::ZERO, 0.5)],
            trace_regions: (0..MAX_GPU_TRACE_REGION_INPUTS as u32)
                .map(|region_id| RenderHybridGiTraceRegion {
                    region_id,
                    bounds_center: Vec3::new(1000.0, 0.0, 0.0),
                    bounds_radius: 0.1,
                    screen_coverage: 1.0,
                    ..Default::default()
                })
                .chain(std::iter::once(RenderHybridGiTraceRegion {
                    region_id: tail_region_id,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 1.0,
                    screen_coverage: 1.0,
                    ..Default::default()
                }))
                .collect(),
            ..Default::default()
        };

        assert_eq!(
            probe_lineage_trace_support_q(None, Some(&extract), &scheduled_trace_region_ids, 100),
            0
        );
    }

    #[test]
    fn probe_resident_ancestors_prefers_runtime_parent_topology_over_legacy_extract() {
        let extract = extract_with_probes(vec![
            probe(100, Vec3::ZERO, 1.0),
            probe(200, Vec3::ZERO, 1.0),
            probe_with_parent(300, 100, Vec3::ZERO, 1.0),
        ]);
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_parent_probes(BTreeMap::from([(300, 200)]))
            .build();

        assert_eq!(
            probe_resident_ancestors(
                Some(&runtime),
                Some(&extract),
                &BTreeSet::from([100, 200]),
                300
            )[0],
            (200, 1)
        );
    }

    #[test]
    fn probe_quantization_prefers_runtime_probe_scene_data_over_legacy_extract() {
        let extract = extract_with_probes(vec![probe(300, Vec3::new(2.0, 4.0, 8.0), 1.5)]);
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                300,
                HybridGiResolveProbeSceneData::new(7, 11, 13, 17),
            )]))
            .build();

        assert_eq!(probe_position_x_q(Some(&runtime), Some(&extract), 300), 7);
        assert_eq!(probe_position_y_q(Some(&runtime), Some(&extract), 300), 11);
        assert_eq!(probe_position_z_q(Some(&runtime), Some(&extract), 300), 13);
        assert_eq!(probe_radius_q(Some(&runtime), Some(&extract), 300), 17);
    }

    #[test]
    fn probe_lineage_trace_support_prefers_runtime_probe_scene_data_over_legacy_extract() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            probes: vec![probe(300, Vec3::new(1000.0, 0.0, 0.0), 0.1)],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 1.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                300,
                HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 96),
            )]))
            .with_trace_region_scene_data(BTreeMap::from([(
                40,
                HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, [255, 128, 64]),
            )]))
            .build();

        assert_eq!(
            probe_lineage_trace_support_q(None, Some(&extract), &[40], 300),
            0
        );
        assert!(
            probe_lineage_trace_support_q(Some(&runtime), Some(&extract), &[40], 300) > 0,
            "runtime-owned probe scene data should be the GPU prepare support geometry when present"
        );
    }

    #[test]
    fn probe_quantization_ignores_legacy_probe_payloads_when_scene_representation_is_budgeted() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            card_budget: 1,
            voxel_budget: 1,
            probes: vec![
                probe(100, Vec3::ZERO, 1.0),
                probe_with_parent(300, 100, Vec3::new(2.0, 0.0, 0.0), 1.5),
            ],
            ..Default::default()
        };

        assert_eq!(probe_position_x_q(None, Some(&extract), 300), 0);
        assert_eq!(probe_radius_q(None, Some(&extract), 300), 0);
        assert_eq!(
            probe_parent_probe_id(None, Some(&extract), 300),
            NO_PARENT_PROBE_ID,
            "scene-representation budgets should keep stale RenderHybridGiProbe parent topology out of GPU prepare quantization"
        );
    }

    #[test]
    fn scheduled_trace_region_ids_ignore_legacy_payloads_when_scene_representation_is_budgeted() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_budget: 2,
            probes: vec![probe(100, Vec3::ZERO, 0.5)],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 1.0,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert_eq!(
            scheduled_live_trace_region_ids(None, Some(&extract), &[40]),
            Vec::<u32>::new()
        );
        assert_eq!(
            probe_lineage_trace_support_q(None, Some(&extract), &[40], 100),
            0,
            "scene-representation budgets should keep old RenderHybridGiTraceRegion schedules from feeding GPU prepare lineage support"
        );
    }

    #[test]
    fn scheduled_runtime_trace_region_ids_ignore_legacy_payloads_when_runtime_has_scene_truth() {
        let legacy_region_id = 40;
        let extract = extract_with_trace_regions(vec![trace_region(legacy_region_id)]);
        let runtime = runtime_scene_truth_with_trace_regions(BTreeMap::from([(
            legacy_region_id,
            runtime_trace_region_scene_data([240, 96, 48]),
        )]));

        assert_eq!(
            scheduled_live_trace_region_ids(Some(&runtime), Some(&extract), &[legacy_region_id]),
            Vec::<u32>::new(),
            "stripped runtime scene truth should keep legacy-backed RenderHybridGiTraceRegion ids out of GPU prepare scheduling"
        );
        assert_eq!(
            probe_lineage_trace_support_q(Some(&runtime), Some(&extract), &[legacy_region_id], 300),
            0,
            "stripped runtime scene truth should prevent legacy-backed runtime trace scene data from feeding GPU prepare lineage support"
        );
    }

    #[test]
    fn scheduled_runtime_trace_region_ids_keep_runtime_only_region_when_legacy_payload_is_scheduled(
    ) {
        let legacy_region_id = 40;
        let runtime_only_region_id = 41;
        let extract = extract_with_trace_regions(vec![trace_region(legacy_region_id)]);
        let runtime = runtime_scene_truth_with_trace_regions(BTreeMap::from([
            (
                legacy_region_id,
                runtime_trace_region_scene_data([240, 96, 48]),
            ),
            (
                runtime_only_region_id,
                runtime_trace_region_scene_data([32, 64, 240]),
            ),
        ]));

        assert_eq!(
            scheduled_live_trace_region_ids(
                Some(&runtime),
                Some(&extract),
                &[legacy_region_id, runtime_only_region_id, runtime_only_region_id],
            ),
            vec![runtime_only_region_id],
            "stripped runtime scene truth should filter only legacy-backed trace ids and keep runtime-only trace scene data"
        );
        assert!(
            probe_lineage_trace_support_q(
                Some(&runtime),
                Some(&extract),
                &[legacy_region_id, runtime_only_region_id],
                300
            ) > 0,
            "runtime-only trace scene data should still feed GPU prepare lineage support"
        );
    }

    fn extract_with_probes(probes: Vec<RenderHybridGiProbe>) -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            probes,
            ..Default::default()
        }
    }

    fn extract_with_trace_regions(
        trace_regions: Vec<RenderHybridGiTraceRegion>,
    ) -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            trace_regions,
            ..Default::default()
        }
    }

    fn runtime_scene_truth_with_trace_regions(
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                300,
                HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 96),
            )]))
            .with_trace_region_scene_data(trace_region_scene_data)
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                300,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.25, 0.45, 0.75], 0.5),
            )]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([300]))
            .build()
    }

    fn runtime_trace_region_scene_data(
        rt_lighting_rgb: [u8; 3],
    ) -> HybridGiResolveTraceRegionSceneData {
        HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, rt_lighting_rgb)
    }

    fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
        RenderHybridGiTraceRegion {
            region_id,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_coverage: 1.0,
            ..Default::default()
        }
    }

    fn probe(probe_id: u32, position: Vec3, radius: f32) -> RenderHybridGiProbe {
        RenderHybridGiProbe {
            probe_id,
            position,
            radius,
            ..Default::default()
        }
    }

    fn probe_with_parent(
        probe_id: u32,
        parent_probe_id: u32,
        position: Vec3,
        radius: f32,
    ) -> RenderHybridGiProbe {
        RenderHybridGiProbe {
            parent_probe_id: Some(parent_probe_id),
            ..probe(probe_id, position, radius)
        }
    }
}
