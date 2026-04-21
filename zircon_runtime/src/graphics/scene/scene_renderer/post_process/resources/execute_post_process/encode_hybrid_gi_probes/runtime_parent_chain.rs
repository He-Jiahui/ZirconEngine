use std::collections::BTreeSet;

use crate::core::framework::render::RenderHybridGiExtract;
use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

const RUNTIME_PARENT_CHAIN_FALLOFF: f32 = 0.82;
const RUNTIME_DESCENDANT_CHAIN_FALLOFF: f32 = 0.84;

pub(super) fn gather_runtime_parent_chain_rgb<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_ancestor: F,
) -> Option<[f32; 4]>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let extract = frame.extract.lighting.hybrid_global_illumination.as_ref()?;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, depth) in parent_probe_chain(extract, probe_id) {
        let Some((rgb, support)) = source_for_ancestor(runtime, ancestor_probe_id) else {
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
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

pub(super) fn gather_runtime_parent_chain_weight(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> Option<f32> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let extract = frame.extract.lighting.hybrid_global_illumination.as_ref()?;

    let mut weighted_weight = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, depth) in parent_probe_chain(extract, probe_id) {
        let Some(weight) = runtime.hierarchy_resolve_weight(ancestor_probe_id) else {
            continue;
        };
        let support = RUNTIME_PARENT_CHAIN_FALLOFF.powi(depth as i32);
        if support <= f32::EPSILON {
            continue;
        }

        weighted_weight += weight * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((weighted_weight / total_support).clamp(0.25, 2.75))
}

pub(super) fn gather_runtime_descendant_chain_rgb<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_descendant: F,
) -> Option<[f32; 4]>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let extract = frame.extract.lighting.hybrid_global_illumination.as_ref()?;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, depth) in descendant_probe_chain(extract, probe_id) {
        let Some((rgb, support)) = source_for_descendant(runtime, descendant_probe_id) else {
            continue;
        };
        let weighted_support = support * RUNTIME_DESCENDANT_CHAIN_FALLOFF.powi((depth - 1) as i32);
        if weighted_support <= f32::EPSILON {
            continue;
        }

        weighted_rgb[0] += rgb[0] * weighted_support;
        weighted_rgb[1] += rgb[1] * weighted_support;
        weighted_rgb[2] += rgb[2] * weighted_support;
        total_support += weighted_support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

pub(super) fn gather_runtime_descendant_chain_weight(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> Option<f32> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let extract = frame.extract.lighting.hybrid_global_illumination.as_ref()?;

    let mut weighted_weight = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, depth) in descendant_probe_chain(extract, probe_id) {
        let Some(weight) = runtime.hierarchy_resolve_weight(descendant_probe_id) else {
            continue;
        };
        let support = RUNTIME_DESCENDANT_CHAIN_FALLOFF.powi((depth - 1) as i32);
        if support <= f32::EPSILON {
            continue;
        }

        weighted_weight += weight * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((weighted_weight / total_support).clamp(0.25, 2.75))
}

pub(super) fn blend_runtime_rgb_lineage_sources(
    exact: Option<[f32; 4]>,
    inherited: Option<[f32; 4]>,
    descendant: Option<[f32; 4]>,
) -> Option<[f32; 4]> {
    let exact = exact.filter(|source| source[3] > f32::EPSILON);
    let inherited = inherited.filter(|source| source[3] > f32::EPSILON);
    let descendant = descendant.filter(|source| source[3] > f32::EPSILON);

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for source in [exact, inherited, descendant].into_iter().flatten() {
        weighted_rgb[0] += source[0] * source[3];
        weighted_rgb[1] += source[1] * source[3];
        weighted_rgb[2] += source[2] * source[3];
        total_support += source[3];
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

pub(super) fn runtime_resolve_weight_support(weight: Option<f32>) -> f32 {
    weight
        .map(|weight| ((weight - 1.0) / 1.75).clamp(0.0, 1.0))
        .unwrap_or(96.0 / 255.0)
}

fn parent_probe_chain(extract: &RenderHybridGiExtract, probe_id: u32) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);
    let mut depth = 0usize;

    while let Some(parent_probe_id) = probe_parent_id(extract, current_probe_id) {
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        depth += 1;
        chain.push((parent_probe_id, depth));
        current_probe_id = parent_probe_id;
    }

    chain
}

fn descendant_probe_chain(extract: &RenderHybridGiExtract, probe_id: u32) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut stack = extract
        .probes
        .iter()
        .filter_map(|probe| {
            (probe.parent_probe_id == Some(probe_id)).then_some((probe.probe_id, 1usize))
        })
        .collect::<Vec<_>>();
    let mut visited_probe_ids = BTreeSet::new();

    while let Some((candidate_probe_id, depth)) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe_id) {
            continue;
        }

        chain.push((candidate_probe_id, depth));
        stack.extend(extract.probes.iter().filter_map(|probe| {
            (probe.parent_probe_id == Some(candidate_probe_id))
                .then_some((probe.probe_id, depth + 1))
        }));
    }

    chain
}

fn probe_parent_id(extract: &RenderHybridGiExtract, probe_id: u32) -> Option<u32> {
    extract
        .probes
        .iter()
        .find(|probe| probe.probe_id == probe_id)
        .and_then(|probe| probe.parent_probe_id)
}
