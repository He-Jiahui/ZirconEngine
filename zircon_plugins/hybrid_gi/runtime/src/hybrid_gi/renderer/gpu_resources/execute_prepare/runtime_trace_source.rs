use std::collections::BTreeSet;

use crate::hybrid_gi::types::HybridGiResolveRuntime;

use super::probe_quantization::pack_rgb8;

pub(super) fn runtime_trace_source(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> (u32, u32, bool) {
    let Some(resolve_runtime) = resolve_runtime else {
        return (0, 0, false);
    };

    if let Some(hierarchy_rt_lighting) = resolve_runtime.hierarchy_rt_lighting(probe_id) {
        let support_q = quantize_support_q(hierarchy_rt_lighting[3]);
        if support_q > 0 {
            return (
                support_q,
                pack_rgb8([
                    quantize_unit_rgb(hierarchy_rt_lighting[0]),
                    quantize_unit_rgb(hierarchy_rt_lighting[1]),
                    quantize_unit_rgb(hierarchy_rt_lighting[2]),
                ]),
                resolve_runtime.hierarchy_rt_lighting_includes_scene_truth(probe_id),
            );
        }
    }

    if let Some(direct_rt_lighting_rgb) = resolve_runtime.probe_rt_lighting_rgb(probe_id) {
        let support_q = resolve_runtime
            .hierarchy_resolve_weight(probe_id)
            .map(runtime_resolve_weight_support_q)
            .unwrap_or(96);
        if support_q > 0 {
            return (support_q, pack_rgb8(direct_rt_lighting_rgb), false);
        }
    }

    runtime_parent_chain_trace_source(resolve_runtime, probe_id)
}

pub(super) fn merge_trace_sources(
    scheduled_trace_support_q: u32,
    scheduled_trace_lighting_rgb: u32,
    runtime_trace_support_q: u32,
    runtime_trace_lighting_rgb: u32,
) -> (u32, u32) {
    let combined_support_q = scheduled_trace_support_q.max(runtime_trace_support_q);

    let combined_rgb = match (
        scheduled_trace_lighting_rgb != 0 && scheduled_trace_support_q > 0,
        runtime_trace_lighting_rgb != 0 && runtime_trace_support_q > 0,
    ) {
        (false, false) => 0,
        (true, false) => scheduled_trace_lighting_rgb,
        (false, true) => runtime_trace_lighting_rgb,
        (true, true) => blend_trace_lighting_rgb(
            scheduled_trace_lighting_rgb,
            scheduled_trace_support_q,
            runtime_trace_lighting_rgb,
            runtime_trace_support_q,
        ),
    };

    (combined_support_q, combined_rgb)
}

pub(super) fn runtime_irradiance_source(
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    probe_id: u32,
) -> (u32, u32, bool) {
    let Some(resolve_runtime) = resolve_runtime else {
        return (0, 0, false);
    };

    if let Some(hierarchy_irradiance) = resolve_runtime.hierarchy_irradiance(probe_id) {
        let support_q = quantize_support_q(hierarchy_irradiance[3]);
        if support_q > 0 {
            return (
                support_q,
                pack_rgb8([
                    quantize_unit_rgb(hierarchy_irradiance[0]),
                    quantize_unit_rgb(hierarchy_irradiance[1]),
                    quantize_unit_rgb(hierarchy_irradiance[2]),
                ]),
                resolve_runtime.hierarchy_irradiance_includes_scene_truth(probe_id),
            );
        }
    }

    runtime_parent_chain_irradiance_source(resolve_runtime, probe_id)
}

fn runtime_resolve_weight_support_q(weight: f32) -> u32 {
    quantize_support_q(((weight - 1.0) / 1.75).clamp(0.0, 1.0))
}

fn runtime_parent_chain_trace_source(
    resolve_runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> (u32, u32, bool) {
    accumulate_parent_chain_runtime_rgb(resolve_runtime, probe_id, |runtime, ancestor_probe_id| {
        if let Some(hierarchy_rt_lighting) = runtime.hierarchy_rt_lighting(ancestor_probe_id) {
            let support = hierarchy_rt_lighting[3];
            return Some((
                [
                    hierarchy_rt_lighting[0],
                    hierarchy_rt_lighting[1],
                    hierarchy_rt_lighting[2],
                ],
                support,
            ));
        }

        runtime.probe_rt_lighting_rgb(ancestor_probe_id).map(|rgb| {
            (
                [
                    rgb[0] as f32 / 255.0,
                    rgb[1] as f32 / 255.0,
                    rgb[2] as f32 / 255.0,
                ],
                runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(ancestor_probe_id)),
            )
        })
    })
}

fn runtime_parent_chain_irradiance_source(
    resolve_runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> (u32, u32, bool) {
    accumulate_parent_chain_runtime_rgb(resolve_runtime, probe_id, |runtime, ancestor_probe_id| {
        runtime
            .hierarchy_irradiance(ancestor_probe_id)
            .map(|hierarchy_irradiance| {
                (
                    [
                        hierarchy_irradiance[0],
                        hierarchy_irradiance[1],
                        hierarchy_irradiance[2],
                    ],
                    hierarchy_irradiance[3],
                )
            })
    })
}

fn accumulate_parent_chain_runtime_rgb<F>(
    resolve_runtime: &HybridGiResolveRuntime,
    probe_id: u32,
    source_for_ancestor: F,
) -> (u32, u32, bool)
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    const RUNTIME_PARENT_CHAIN_FALLOFF: f32 = 0.82;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, depth) in parent_probe_chain(resolve_runtime, probe_id) {
        let Some((rgb, support)) = source_for_ancestor(resolve_runtime, ancestor_probe_id) else {
            continue;
        };
        let weighted_support = support * RUNTIME_PARENT_CHAIN_FALLOFF.powi(depth as i32);
        if weighted_support <= f32::EPSILON {
            continue;
        }

        weighted_rgb[0] += rgb[0] * weighted_support;
        weighted_rgb[1] += rgb[1] * weighted_support;
        weighted_rgb[2] += rgb[2] * weighted_support;
        total_support += weighted_support;
    }

    if total_support <= f32::EPSILON {
        return (0, 0, false);
    }

    (
        quantize_support_q(total_support),
        pack_rgb8([
            quantize_unit_rgb(weighted_rgb[0] / total_support),
            quantize_unit_rgb(weighted_rgb[1] / total_support),
            quantize_unit_rgb(weighted_rgb[2] / total_support),
        ]),
        false,
    )
}

fn runtime_resolve_weight_support(weight: Option<f32>) -> f32 {
    weight
        .map(runtime_resolve_weight_support_q)
        .map(|support_q| support_q as f32 / 255.0)
        .unwrap_or(96.0 / 255.0)
}

fn quantize_support_q(weight: f32) -> u32 {
    (weight.clamp(0.0, 1.0) * 255.0).round() as u32
}

fn quantize_unit_rgb(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn blend_trace_lighting_rgb(
    scheduled_trace_lighting_rgb: u32,
    scheduled_trace_support_q: u32,
    runtime_trace_lighting_rgb: u32,
    runtime_trace_support_q: u32,
) -> u32 {
    let scheduled_rgb = unpack_rgb8(scheduled_trace_lighting_rgb);
    let runtime_rgb = unpack_rgb8(runtime_trace_lighting_rgb);
    let total_weight = scheduled_trace_support_q.max(1) + runtime_trace_support_q.max(1);

    pack_rgb8([
        blend_trace_channel(
            scheduled_rgb[0],
            scheduled_trace_support_q.max(1),
            runtime_rgb[0],
            runtime_trace_support_q.max(1),
            total_weight,
        ),
        blend_trace_channel(
            scheduled_rgb[1],
            scheduled_trace_support_q.max(1),
            runtime_rgb[1],
            runtime_trace_support_q.max(1),
            total_weight,
        ),
        blend_trace_channel(
            scheduled_rgb[2],
            scheduled_trace_support_q.max(1),
            runtime_rgb[2],
            runtime_trace_support_q.max(1),
            total_weight,
        ),
    ])
}

fn blend_trace_channel(
    scheduled: u8,
    scheduled_weight: u32,
    runtime: u8,
    runtime_weight: u32,
    total_weight: u32,
) -> u8 {
    (((u32::from(scheduled) * scheduled_weight + u32::from(runtime) * runtime_weight)
        + total_weight / 2)
        / total_weight) as u8
}

fn unpack_rgb8(packed: u32) -> [u8; 3] {
    [
        (packed & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        ((packed >> 16) & 0xff) as u8,
    ]
}

fn parent_probe_chain(runtime: &HybridGiResolveRuntime, probe_id: u32) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);
    let mut depth = 0usize;

    while let Some(parent_probe_id) = runtime.parent_probe_id(current_probe_id) {
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        depth += 1;
        chain.push((parent_probe_id, depth));
        current_probe_id = parent_probe_id;
    }

    chain
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn runtime_irradiance_source_follows_runtime_parent_topology() {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_parent_probes(BTreeMap::from([(300, 200)]))
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([
                (
                    100,
                    HybridGiResolveRuntime::pack_rgb_and_weight([1.0, 0.0, 0.0], 1.0),
                ),
                (
                    200,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.0, 0.0, 1.0], 1.0),
                ),
            ]))
            .build();

        let (support_q, rgb, scene_truth) = runtime_irradiance_source(Some(&runtime), 300);

        assert!(support_q > 0);
        assert_eq!(unpack_rgb8(rgb), [0, 0, 255]);
        assert!(!scene_truth);
    }

    #[test]
    fn runtime_trace_source_does_not_walk_parent_lineage_when_runtime_topology_is_flat() {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_rt_lighting_rgb(BTreeMap::from([(100, [240, 96, 48])]))
            .build();

        assert_eq!(runtime_trace_source(Some(&runtime), 300), (0, 0, false));
    }

    #[test]
    fn runtime_trace_source_breaks_runtime_parent_cycles() {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_parent_probes(BTreeMap::from([(200, 300), (300, 200)]))
            .with_probe_rt_lighting_rgb(BTreeMap::from([(200, [12, 24, 240])]))
            .build();

        let (support_q, rgb, scene_truth) = runtime_trace_source(Some(&runtime), 300);

        assert!(support_q > 0);
        assert_eq!(unpack_rgb8(rgb), [12, 24, 240]);
        assert!(!scene_truth);
    }
}
