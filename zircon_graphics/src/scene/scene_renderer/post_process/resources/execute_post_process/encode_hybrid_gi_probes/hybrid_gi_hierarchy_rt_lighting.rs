use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::{RenderHybridGiProbe, RenderHybridGiTraceRegion};

use crate::types::EditorOrRuntimeFrame;

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;

const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;

pub(super) fn hybrid_gi_hierarchy_rt_lighting(
    frame: &EditorOrRuntimeFrame,
    source: &RenderHybridGiProbe,
) -> [f32; 4] {
    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return [0.0; 4];
    };
    let prepare = frame.hybrid_gi_prepare.as_ref();
    let scheduled_trace_region_ids = prepare
        .map(|prepare| prepare.scheduled_trace_region_ids.as_slice())
        .unwrap_or(&[]);

    let probes_by_id = extract
        .probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();
    let trace_regions_by_id = extract
        .trace_regions
        .iter()
        .copied()
        .map(|region| (region.region_id, region))
        .collect::<BTreeMap<_, _>>();
    let resident_prepare_by_id = prepare
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if let Some(runtime_rt_lighting) =
        runtime_hierarchy_rt_lighting(frame, source, &resident_prepare_by_id)
    {
        return runtime_rt_lighting;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if scheduled_trace_region_ids.is_empty() {
        if total_support <= f32::EPSILON {
            return [0.0; 4];
        }
        return [
            weighted_rgb[0] / total_support,
            weighted_rgb[1] / total_support,
            weighted_rgb[2] / total_support,
            total_support.clamp(0.0, 0.75),
        ];
    }

    let mut current_probe_id = source.probe_id;
    let mut visited_probe_ids = BTreeSet::from([source.probe_id]);
    let mut ancestor_depth = 0usize;

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
        for region_id in scheduled_trace_region_ids {
            let Some(region) = trace_regions_by_id.get(region_id) else {
                continue;
            };
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
        return [0.0; 4];
    }

    let inherited_weight = (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75);
    [
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        inherited_weight,
    ]
}

fn runtime_hierarchy_rt_lighting(
    frame: &EditorOrRuntimeFrame,
    source: &RenderHybridGiProbe,
    resident_prepare_by_id: &BTreeMap<u32, &crate::types::HybridGiPrepareProbe>,
) -> Option<[f32; 4]> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let direct_rt_lighting_rgb = runtime.probe_rt_lighting_rgb.get(&source.probe_id).copied();
    let hierarchy_rt_lighting = runtime.hierarchy_rt_lighting(source.probe_id);
    if direct_rt_lighting_rgb.is_none() && hierarchy_rt_lighting.is_none() {
        return None;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if let Some(direct_rt_lighting_rgb) = direct_rt_lighting_rgb {
        let direct_support = resident_prepare_by_id
            .get(&source.probe_id)
            .map(|probe| (0.25 + hybrid_gi_budget_weight(probe.ray_budget) * 0.5).clamp(0.25, 0.75))
            .unwrap_or(0.3);
        weighted_rgb[0] += direct_rt_lighting_rgb[0] as f32 / 255.0 * direct_support;
        weighted_rgb[1] += direct_rt_lighting_rgb[1] as f32 / 255.0 * direct_support;
        weighted_rgb[2] += direct_rt_lighting_rgb[2] as f32 / 255.0 * direct_support;
        total_support += direct_support;
    }
    if let Some(hierarchy_rt_lighting) = hierarchy_rt_lighting {
        if hierarchy_rt_lighting[3] > f32::EPSILON {
            weighted_rgb[0] += hierarchy_rt_lighting[0] * hierarchy_rt_lighting[3];
            weighted_rgb[1] += hierarchy_rt_lighting[1] * hierarchy_rt_lighting[3];
            weighted_rgb[2] += hierarchy_rt_lighting[2] * hierarchy_rt_lighting[3];
            total_support += hierarchy_rt_lighting[3];
        }
    }

    if total_support <= f32::EPSILON {
        return Some([0.0; 4]);
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

fn hierarchy_trace_region_support(
    probe: &RenderHybridGiProbe,
    region: &RenderHybridGiTraceRegion,
) -> f32 {
    let reach = (probe.radius.max(0.05) + region.bounds_radius.max(0.05)).max(0.05);
    let distance = probe.position.distance(region.bounds_center);
    let falloff = (1.0 - distance / reach).max(0.0);
    let coverage_weight = (0.35 + region.screen_coverage.clamp(0.0, 1.0) * 0.65).clamp(0.35, 1.0);
    falloff * falloff * coverage_weight
}

fn hybrid_gi_trace_region_rt_lighting(region: &RenderHybridGiTraceRegion) -> [f32; 4] {
    let rgb = [
        region.rt_lighting_rgb[0] as f32 / 255.0,
        region.rt_lighting_rgb[1] as f32 / 255.0,
        region.rt_lighting_rgb[2] as f32 / 255.0,
    ];
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]);

    [rgb[0], rgb[1], rgb[2], max_component]
}
