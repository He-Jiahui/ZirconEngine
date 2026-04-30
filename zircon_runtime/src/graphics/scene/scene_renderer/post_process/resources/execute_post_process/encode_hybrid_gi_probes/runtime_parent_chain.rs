use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderHybridGiExtract;
use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::hybrid_gi_trace_region_source::fallback_trace_region_sources_by_id;
use super::hybrid_gi_probe_source::{fallback_probe_sources_by_id, HybridGiProbeSource};

const RUNTIME_PARENT_CHAIN_FALLOFF: f32 = 0.82;
const RUNTIME_DESCENDANT_CHAIN_FALLOFF: f32 = 0.84;
const RUNTIME_PARENT_CHAIN_REVISION_SEED: u32 = 0x9E37_79B9;
const RUNTIME_DESCENDANT_CHAIN_REVISION_SEED: u32 = 0x85EB_CA77;

pub(super) fn gather_runtime_parent_chain_rgb<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_ancestor: F,
) -> Option<[f32; 4]>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, depth) in parent_probe_chain(frame, probe_id)? {
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

pub(super) fn gather_runtime_parent_chain_rgb_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_ancestor: F,
) -> Option<[f32; 4]>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, _) in parent_probe_chain(frame, probe_id)? {
        let Some((rgb, support)) = source_for_ancestor(runtime, ancestor_probe_id) else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        weighted_rgb[0] += rgb[0] * support;
        weighted_rgb[1] += rgb[1] * support;
        weighted_rgb[2] += rgb[2] * support;
        total_support += support;
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

pub(super) fn gather_runtime_parent_chain_support_and_quality_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_ancestor: F,
) -> Option<(f32, f32, f32)>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<(f32, f32, f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_quality = 0.0_f32;
    let mut weighted_freshness = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, _) in parent_probe_chain(frame, probe_id)? {
        let Some((support, quality, freshness)) = source_for_ancestor(runtime, ancestor_probe_id)
        else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        total_support += support;
        weighted_quality += quality.clamp(0.0, 1.0) * support;
        weighted_freshness += freshness.clamp(0.0, 1.0) * support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((
        total_support.clamp(0.0, 0.75),
        (weighted_quality / total_support).clamp(0.0, 1.0),
        (weighted_freshness / total_support).clamp(0.0, 1.0),
    ))
}

pub(super) fn gather_runtime_parent_chain_support_and_revision_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_ancestor: F,
) -> Option<(f32, u32)>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<(f32, u32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut total_support = 0.0_f32;
    let mut mixed_revision = 0u32;
    let mut has_revision = false;
    for (ancestor_probe_id, _) in parent_probe_chain(frame, probe_id)? {
        let Some((support, revision)) = source_for_ancestor(runtime, ancestor_probe_id) else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        let support_q = quantize_revision_support(support);
        let packed_revision = revision ^ support_q.rotate_left(8);
        mixed_revision = mix_lineage_scene_truth_revision(
            mixed_revision,
            packed_revision,
            has_revision,
            RUNTIME_PARENT_CHAIN_REVISION_SEED,
        );
        total_support += support;
        has_revision = true;
    }

    (total_support > f32::EPSILON && has_revision)
        .then_some((total_support.clamp(0.0, 0.75), mixed_revision))
}

pub(super) fn gather_runtime_parent_chain_weight(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> Option<f32> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_weight = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (ancestor_probe_id, depth) in parent_probe_chain(frame, probe_id)? {
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

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, depth) in descendant_probe_chain(frame, probe_id)? {
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

pub(super) fn gather_runtime_descendant_chain_rgb_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_descendant: F,
) -> Option<[f32; 4]>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<([f32; 3], f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, _) in descendant_probe_chain(frame, probe_id)? {
        let Some((rgb, support)) = source_for_descendant(runtime, descendant_probe_id) else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        weighted_rgb[0] += rgb[0] * support;
        weighted_rgb[1] += rgb[1] * support;
        weighted_rgb[2] += rgb[2] * support;
        total_support += support;
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

pub(super) fn gather_runtime_descendant_chain_support_and_quality_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_descendant: F,
) -> Option<(f32, f32, f32)>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<(f32, f32, f32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_quality = 0.0_f32;
    let mut weighted_freshness = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, _) in descendant_probe_chain(frame, probe_id)? {
        let Some((support, quality, freshness)) =
            source_for_descendant(runtime, descendant_probe_id)
        else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        total_support += support;
        weighted_quality += quality.clamp(0.0, 1.0) * support;
        weighted_freshness += freshness.clamp(0.0, 1.0) * support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((
        total_support.clamp(0.0, 0.75),
        (weighted_quality / total_support).clamp(0.0, 1.0),
        (weighted_freshness / total_support).clamp(0.0, 1.0),
    ))
}

pub(super) fn gather_runtime_descendant_chain_support_and_revision_without_depth_falloff<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_descendant: F,
) -> Option<(f32, u32)>
where
    F: Fn(&HybridGiResolveRuntime, u32) -> Option<(f32, u32)>,
{
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut total_support = 0.0_f32;
    let mut mixed_revision = 0u32;
    let mut has_revision = false;
    for (descendant_probe_id, _) in descendant_probe_chain(frame, probe_id)? {
        let Some((support, revision)) = source_for_descendant(runtime, descendant_probe_id) else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        let support_q = quantize_revision_support(support);
        let packed_revision = revision ^ support_q.rotate_left(8);
        mixed_revision = mix_lineage_scene_truth_revision(
            mixed_revision,
            packed_revision,
            has_revision,
            RUNTIME_DESCENDANT_CHAIN_REVISION_SEED,
        );
        total_support += support;
        has_revision = true;
    }

    (total_support > f32::EPSILON && has_revision)
        .then_some((total_support.clamp(0.0, 0.75), mixed_revision))
}

pub(super) fn gather_runtime_descendant_chain_weight(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> Option<f32> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;

    let mut weighted_weight = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (descendant_probe_id, depth) in descendant_probe_chain(frame, probe_id)? {
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

pub(super) fn runtime_irradiance_lineage_has_scene_truth(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> bool {
    runtime_lineage_has_scene_truth(frame, probe_id, runtime_probe_has_irradiance_scene_truth)
}

pub(super) fn runtime_rt_lighting_lineage_has_scene_truth(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> bool {
    runtime_lineage_has_scene_truth(frame, probe_id, runtime_probe_has_rt_lighting_scene_truth)
}

pub(in super::super) fn frame_has_runtime_probe_lineage_scene_truth(
    frame: &ViewportRenderFrame,
) -> bool {
    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return false;
    };

    prepare
        .resident_probes
        .iter()
        .any(|resident_probe| runtime_probe_lineage_has_scene_truth(frame, resident_probe.probe_id))
}

pub(in super::super) fn frame_has_runtime_scene_truth(frame: &ViewportRenderFrame) -> bool {
    let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() else {
        return false;
    };

    runtime
        .scene_truth_irradiance_probe_ids()
        .any(|probe_id| runtime_probe_has_irradiance_scene_truth(runtime, probe_id))
        || runtime
            .scene_truth_rt_lighting_probe_ids()
            .any(|probe_id| runtime_probe_has_rt_lighting_scene_truth(runtime, probe_id))
}

pub(in super::super) fn runtime_probe_lineage_has_scene_truth(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> bool {
    runtime_irradiance_lineage_has_scene_truth(frame, probe_id)
        || runtime_rt_lighting_lineage_has_scene_truth(frame, probe_id)
}

fn runtime_lineage_has_scene_truth<F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    probe_has_scene_truth: F,
) -> bool
where
    F: Fn(&HybridGiResolveRuntime, u32) -> bool,
{
    let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() else {
        return false;
    };
    if probe_has_scene_truth(runtime, probe_id) {
        return true;
    }

    let Some(parent_chain) = parent_probe_chain(frame, probe_id) else {
        return false;
    };
    let Some(descendant_chain) = descendant_probe_chain(frame, probe_id) else {
        return false;
    };
    parent_chain
        .into_iter()
        .chain(descendant_chain)
        .any(|(lineage_probe_id, _)| probe_has_scene_truth(runtime, lineage_probe_id))
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
            || (runtime.has_probe_rt_lighting(probe_id)
                && runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(probe_id))
                    > f32::EPSILON))
}

fn quantize_revision_support(value: f32) -> u32 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u32
}

fn mix_lineage_scene_truth_revision(
    mixed_revision: u32,
    packed_revision: u32,
    has_revision: bool,
    chain_seed: u32,
) -> u32 {
    if has_revision {
        mix_revision_words(mixed_revision, packed_revision.rotate_left(5))
    } else {
        mix_revision_words(chain_seed, packed_revision.rotate_left(5))
    }
}

fn mix_revision_words(left: u32, right: u32) -> u32 {
    let mut mixed = left.wrapping_add(0x7FEB_352D).wrapping_mul(0x846C_A68B);
    mixed ^= right.rotate_left(16);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x2C1B_3C6D);
    mixed ^ (mixed >> 12)
}

pub(super) fn runtime_resolve_weight_support(weight: Option<f32>) -> f32 {
    weight
        .map(|weight| ((weight - 1.0) / 1.75).clamp(0.0, 1.0))
        .unwrap_or(96.0 / 255.0)
}

pub(super) fn runtime_parent_topology_is_authoritative(frame: &ViewportRenderFrame) -> bool {
    frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(HybridGiResolveRuntime::has_parent_topology)
        .unwrap_or(false)
        || frame_has_runtime_scene_truth(frame)
}

#[cfg(test)]
pub(super) fn frame_has_scheduled_trace_region_payload(frame: &ViewportRenderFrame) -> bool {
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return false;
    }

    !scheduled_live_trace_region_ids(frame).is_empty()
}

pub(super) fn frame_has_scheduled_trace_region_source(frame: &ViewportRenderFrame) -> bool {
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return !scheduled_runtime_trace_region_ids(frame).is_empty();
    }

    !scheduled_live_trace_region_ids(frame).is_empty()
}

pub(in super::super) fn scheduled_runtime_trace_region_ids(
    frame: &ViewportRenderFrame,
) -> Vec<u32> {
    let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() else {
        return Vec::new();
    };
    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return Vec::new();
    };
    let scene_prepare_extract_trace_region_ids =
        if frame.hybrid_gi_scene_prepare.is_some() || frame_has_runtime_scene_truth(frame) {
            current_extract_trace_region_source_ids(frame)
        } else {
            BTreeSet::new()
        };

    let mut scheduled_region_ids = BTreeSet::new();
    prepare
        .scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .filter(|region_id| !scene_prepare_extract_trace_region_ids.contains(region_id))
        .filter(|region_id| runtime.trace_region_scene_data(*region_id).is_some())
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
        .collect()
}

fn current_extract_trace_region_source_ids(frame: &ViewportRenderFrame) -> BTreeSet<u32> {
    fallback_trace_region_sources_by_id(frame.extract.lighting.hybrid_global_illumination.as_ref())
        .into_keys()
        .collect()
}

pub(in super::super) fn scheduled_live_trace_region_ids(frame: &ViewportRenderFrame) -> Vec<u32> {
    if frame.hybrid_gi_resolve_runtime.is_some() {
        return Vec::new();
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return Vec::new();
    };
    let trace_region_ids = fallback_trace_region_sources_by_id(
        frame.extract.lighting.hybrid_global_illumination.as_ref(),
    )
    .into_keys()
    .collect::<BTreeSet<_>>();
    let mut scheduled_region_ids = BTreeSet::new();

    prepare
        .scheduled_trace_region_ids
        .iter()
        .copied()
        .filter(|region_id| scheduled_region_ids.insert(*region_id))
        .filter(|region_id| trace_region_ids.contains(region_id))
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
        .collect()
}

pub(super) fn temporal_parent_probe_chain(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    extract_parent_probe_id: Option<u32>,
) -> Vec<(u32, usize)> {
    if let Some(chain) = parent_probe_chain(frame, probe_id) {
        return chain;
    }

    if frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .is_some_and(|extract| !extract.enabled)
    {
        return Vec::new();
    }

    extract_parent_probe_id
        .map(|parent_probe_id| vec![(parent_probe_id, 1)])
        .unwrap_or_default()
}

fn parent_probe_chain(frame: &ViewportRenderFrame, probe_id: u32) -> Option<Vec<(u32, usize)>> {
    if let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() {
        return Some(parent_probe_chain_from_runtime(runtime, probe_id));
    }

    if frame_hybrid_gi_extract_is_enabled(frame) {
        return Some(parent_probe_chain_from_parent_map(
            &current_extract_probe_parent_probes(frame),
            probe_id,
        ));
    }
    frame.hybrid_gi_resolve_runtime.as_ref().map(|_| Vec::new())
}

fn parent_probe_chain_from_runtime(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    runtime.parent_probe_chain(probe_id)
}

fn parent_probe_chain_from_parent_map(
    probe_parent_probes: &BTreeMap<u32, u32>,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);
    let mut depth = 0usize;

    while let Some(parent_probe_id) = probe_parent_probes.get(&current_probe_id).copied() {
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        depth += 1;
        chain.push((parent_probe_id, depth));
        current_probe_id = parent_probe_id;
    }

    chain
}

fn descendant_probe_chain(frame: &ViewportRenderFrame, probe_id: u32) -> Option<Vec<(u32, usize)>> {
    if let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() {
        return Some(descendant_probe_chain_from_runtime(runtime, probe_id));
    }

    if let Some(extract) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    {
        let extract_chain = descendant_probe_chain_from_extract(extract, probe_id);
        if !extract_chain.is_empty() {
            return Some(extract_chain);
        }
    }
    frame.hybrid_gi_resolve_runtime.as_ref().map(|_| Vec::new())
}

fn descendant_probe_chain_from_runtime(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    runtime.descendant_probe_chain(probe_id)
}

fn descendant_probe_chain_from_extract(
    extract: &RenderHybridGiExtract,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    descendant_probe_chain_from_parent_map(&probe_parent_probes_from_extract(extract), probe_id)
}

fn descendant_probe_chain_from_parent_map(
    probe_parent_probes: &BTreeMap<u32, u32>,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut stack = probe_parent_probes
        .iter()
        .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
            (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
        })
        .collect::<Vec<_>>();
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    while let Some((candidate_probe_id, depth)) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe_id) {
            continue;
        }

        chain.push((candidate_probe_id, depth));
        stack.extend(probe_parent_probes.iter().filter_map(
            |(&grandchild_probe_id, &parent_probe_id)| {
                (parent_probe_id == candidate_probe_id).then_some((grandchild_probe_id, depth + 1))
            },
        ));
    }

    chain
}

fn frame_hybrid_gi_extract_is_enabled(frame: &ViewportRenderFrame) -> bool {
    frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .is_some_and(|extract| extract.enabled)
}

fn current_extract_probe_parent_probes(frame: &ViewportRenderFrame) -> BTreeMap<u32, u32> {
    frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .map(probe_parent_probes_from_extract)
        .unwrap_or_default()
}

fn probe_parent_probes_from_extract(extract: &RenderHybridGiExtract) -> BTreeMap<u32, u32> {
    fallback_probe_sources_by_id(Some(extract))
        .into_values()
        .filter_map(|probe| {
            probe
                .parent_probe_id()
                .map(|parent_probe_id| (probe.probe_id(), parent_probe_id))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::types::{
        HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
    };

    #[test]
    fn parent_probe_chain_blocks_legacy_extract_lineage_when_flat_runtime_scene_truth_has_legacy_rt_source(
    ) {
        let frame = frame_with_flat_runtime_scene_truth_and_legacy_extract_lineage();

        assert_eq!(
            super::parent_probe_chain(&frame, 300),
            Some(Vec::new()),
            "expected flat runtime scene truth to keep legacy RenderHybridGiProbe parent links out of runtime parent-chain fallback even when legacy RT continuation data is present"
        );
    }

    #[test]
    fn descendant_probe_chain_blocks_legacy_extract_lineage_when_flat_runtime_scene_truth_has_legacy_rt_source(
    ) {
        let frame = frame_with_flat_runtime_scene_truth_and_legacy_extract_lineage();

        assert_eq!(
            super::descendant_probe_chain(&frame, 400),
            Some(Vec::new()),
            "expected flat runtime scene truth to keep legacy RenderHybridGiProbe child links out of runtime descendant-chain fallback even when legacy RT continuation data is present"
        );
    }

    #[test]
    fn parent_probe_chain_blocks_legacy_extract_lineage_when_runtime_is_flat() {
        let frame = frame_with_flat_runtime_and_legacy_extract_lineage();

        assert_eq!(
            super::parent_probe_chain(&frame, 300),
            Some(Vec::new()),
            "expected flat runtime topology to keep legacy RenderHybridGiProbe parent links out of runtime parent-chain fallback"
        );
    }

    #[test]
    fn descendant_probe_chain_blocks_legacy_extract_lineage_when_runtime_is_flat() {
        let frame = frame_with_flat_runtime_and_legacy_extract_lineage();

        assert_eq!(
            super::descendant_probe_chain(&frame, 400),
            Some(Vec::new()),
            "expected flat runtime topology to keep legacy RenderHybridGiProbe child links out of runtime descendant-chain fallback"
        );
    }

    #[test]
    fn flat_runtime_blocks_legacy_scheduled_trace_region_payload_presence() {
        let frame = frame_with_flat_runtime_and_legacy_scheduled_trace_region();

        assert_eq!(
            super::scheduled_live_trace_region_ids(&frame),
            Vec::<u32>::new(),
            "expected flat runtime ownership to stop legacy RenderHybridGiTraceRegion schedule ids before any main-path caller can consume them"
        );
        assert!(
            !super::frame_has_scheduled_trace_region_payload(&frame),
            "expected flat runtime ownership to stop legacy RenderHybridGiTraceRegion schedule presence from influencing probe hierarchy or temporal fallback selection"
        );
    }

    #[test]
    fn runtime_trace_region_scene_data_counts_as_trace_region_source_without_legacy_payload_presence(
    ) {
        let frame = frame_with_runtime_trace_region_scene_data();

        assert_eq!(
            super::scheduled_runtime_trace_region_ids(&frame),
            vec![40],
            "expected runtime trace-region scene data to provide the scheduled trace-region source"
        );
        assert!(
            !super::frame_has_scheduled_trace_region_payload(&frame),
            "expected runtime trace-region scene data not to reclassify as legacy RenderHybridGiTraceRegion payload presence"
        );
        assert!(
            super::frame_has_scheduled_trace_region_source(&frame),
            "expected runtime trace-region scene data to participate in runtime-aware hierarchy and temporal trace-source checks"
        );
    }

    #[test]
    fn scene_prepare_runtime_trace_region_source_ignores_runtime_data_backed_by_legacy_trace_payload(
    ) {
        let frame =
            frame_with_scene_prepare_runtime_trace_region_scene_data_backed_by_legacy_payload();

        assert_eq!(
            super::scheduled_runtime_trace_region_ids(&frame),
            Vec::<u32>::new(),
            "expected scene-prepare runtime trace-region source selection to reject runtime scene data that was still backed by the old RenderHybridGiTraceRegion payload path"
        );
        assert!(
            !super::frame_has_scheduled_trace_region_source(&frame),
            "expected legacy-backed runtime trace-region scene data not to keep hierarchy or temporal compatibility paths alive during scene-prepare"
        );
    }

    #[test]
    fn scene_prepare_runtime_trace_region_source_keeps_runtime_only_region_when_legacy_payload_is_scheduled(
    ) {
        let frame =
            frame_with_scene_prepare_runtime_trace_region_scene_data_and_mixed_legacy_payload();

        assert_eq!(
            super::scheduled_runtime_trace_region_ids(&frame),
            vec![41],
            "expected scene-prepare runtime trace-region source selection to filter only legacy-backed ids and keep runtime-only trace scene data"
        );
        assert!(
            super::frame_has_scheduled_trace_region_source(&frame),
            "expected the runtime-only trace-region scene data to remain visible to runtime-aware hierarchy and temporal checks"
        );
    }

    #[test]
    fn stripped_runtime_trace_region_source_ignores_runtime_data_backed_by_legacy_trace_payload() {
        let frame = frame_with_stripped_runtime_trace_region_scene_data_backed_by_legacy_payload();

        assert_eq!(
            super::scheduled_runtime_trace_region_ids(&frame),
            Vec::<u32>::new(),
            "expected stripped runtime scene truth to reject runtime trace scene data still backed by the old RenderHybridGiTraceRegion payload path"
        );
        assert!(
            !super::frame_has_scheduled_trace_region_source(&frame),
            "expected legacy-backed runtime trace-region scene data not to keep compatibility trace-source paths alive after scene-prepare has been stripped"
        );
    }

    #[test]
    fn stripped_runtime_trace_region_source_keeps_runtime_only_region_when_legacy_payload_is_scheduled(
    ) {
        let frame = frame_with_stripped_runtime_trace_region_scene_data_and_mixed_legacy_payload();

        assert_eq!(
            super::scheduled_runtime_trace_region_ids(&frame),
            vec![41],
            "expected stripped runtime scene truth to filter only legacy-backed trace ids and keep runtime-only trace scene data"
        );
        assert!(
            super::frame_has_scheduled_trace_region_source(&frame),
            "expected stripped runtime-only trace-region scene data to remain visible to runtime-aware hierarchy and temporal checks"
        );
    }

    #[test]
    fn scene_representation_budget_blocks_legacy_scheduled_trace_region_payload_presence() {
        let frame = frame_with_budgeted_scene_representation_and_legacy_scheduled_trace_region();

        assert_eq!(
            super::scheduled_live_trace_region_ids(&frame),
            Vec::<u32>::new(),
            "expected budgeted scene-representation extracts to stop legacy RenderHybridGiTraceRegion schedule ids before any post-process caller can consume them"
        );
        assert!(
            !super::frame_has_scheduled_trace_region_payload(&frame),
            "expected budgeted scene-representation extracts to stop legacy RenderHybridGiTraceRegion schedule presence from influencing compatibility fallback selection"
        );
    }

    #[test]
    fn scene_representation_budget_blocks_legacy_extract_lineage() {
        let frame = frame_with_budgeted_scene_representation_and_legacy_extract_lineage();

        assert_eq!(
            super::parent_probe_chain(&frame, 300),
            Some(Vec::new()),
            "expected budgeted scene-representation extracts to keep legacy RenderHybridGiProbe parent links out of runtime parent-chain fallback"
        );
        assert_eq!(
            super::descendant_probe_chain(&frame, 400),
            None,
            "expected budgeted scene-representation extracts to leave no legacy RenderHybridGiProbe descendant-chain source"
        );
    }

    #[test]
    fn descendant_probe_chain_from_extract_uses_first_legacy_probe_payload_for_duplicate_ids() {
        let extract = RenderHybridGiExtract {
            enabled: true,
            probes: vec![
                RenderHybridGiProbe {
                    probe_id: 300,
                    parent_probe_id: None,
                    ..Default::default()
                },
                RenderHybridGiProbe {
                    probe_id: 300,
                    parent_probe_id: Some(400),
                    ..Default::default()
                },
                RenderHybridGiProbe {
                    probe_id: 400,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            super::descendant_probe_chain_from_extract(&extract, 400),
            Vec::<(u32, usize)>::new(),
            "expected legacy descendant fallback to ignore duplicate RenderHybridGiProbe payloads after the first live id, matching runtime registration"
        );
    }

    #[test]
    fn descendant_probe_chain_from_extract_does_not_return_origin_when_legacy_parent_cycle_exists()
    {
        let extract = RenderHybridGiExtract {
            enabled: true,
            probes: vec![
                RenderHybridGiProbe {
                    probe_id: 300,
                    parent_probe_id: Some(400),
                    ..Default::default()
                },
                RenderHybridGiProbe {
                    probe_id: 400,
                    parent_probe_id: Some(300),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            super::descendant_probe_chain_from_extract(&extract, 300),
            Vec::<(u32, usize)>::new(),
            "expected legacy descendant fallback to break cyclic parent edges before traversal instead of returning the origin probe as its own descendant"
        );
    }

    #[test]
    fn descendant_probe_chain_from_runtime_does_not_return_origin_when_runtime_parent_cycle_exists()
    {
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_parent_probes(BTreeMap::from([(300, 400), (400, 300)]))
            .build();

        assert_eq!(
            super::descendant_probe_chain_from_runtime(&runtime, 300),
            vec![(400, 1)],
            "expected runtime descendant fallback to stop at the cycle boundary instead of returning the origin probe as its own descendant"
        );
    }

    #[test]
    fn temporal_parent_probe_chain_does_not_restore_sanitized_legacy_parent_id() {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 1,
            probes: vec![child_probe],
            ..Default::default()
        });
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32));

        assert_eq!(
            super::temporal_parent_probe_chain(
                &frame,
                child_probe_id,
                child_probe.parent_probe_id
            ),
            Vec::<(u32, usize)>::new(),
            "expected temporal parent-chain fallback to preserve extract sanitization instead of reintroducing a stale RenderHybridGiProbe parent id without a live payload"
        );
    }

    #[test]
    fn temporal_parent_probe_chain_does_not_restore_disabled_legacy_parent_id() {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: false,
            probe_budget: 1,
            probes: vec![child_probe],
            ..Default::default()
        });
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32));

        assert_eq!(
            super::temporal_parent_probe_chain(
                &frame,
                child_probe_id,
                child_probe.parent_probe_id
            ),
            Vec::<(u32, usize)>::new(),
            "expected disabled RenderHybridGiProbe compatibility payloads not to re-enter temporal parent-chain fallback through a stale caller-provided parent id"
        );
    }

    fn frame_with_flat_runtime_scene_truth_and_legacy_extract_lineage() -> ViewportRenderFrame {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 2,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                        stale_parent_probe_id,
                        HybridGiResolveRuntime::pack_rgb_and_weight([0.68, 0.08, 0.06], 0.58),
                    )]))
                    .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([
                        stale_parent_probe_id,
                    ]))
                    .with_probe_rt_lighting_rgb(BTreeMap::from([(child_probe_id, [240, 96, 48])]))
                    .build(),
            ))
    }

    fn frame_with_flat_runtime_and_legacy_extract_lineage() -> ViewportRenderFrame {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 2,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()))
    }

    fn frame_with_flat_runtime_and_legacy_scheduled_trace_region() -> ViewportRenderFrame {
        let trace_region_id = 40;
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(trace_region_id),
                region_id: trace_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(crate::graphics::types::HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![trace_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()))
    }

    fn frame_with_runtime_trace_region_scene_data() -> ViewportRenderFrame {
        let trace_region_id = 40;
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![trace_region_id, trace_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(BTreeMap::from([(
                        trace_region_id,
                        HybridGiResolveTraceRegionSceneData::new(
                            2048,
                            2048,
                            2048,
                            96,
                            128,
                            [32, 64, 240],
                        ),
                    )]))
                    .build(),
            ))
    }

    fn frame_with_scene_prepare_runtime_trace_region_scene_data_backed_by_legacy_payload(
    ) -> ViewportRenderFrame {
        let trace_region_id = 40;
        let extract = extract_with_legacy_trace_region(trace_region_id);

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![trace_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame::default()))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(BTreeMap::from([(
                        trace_region_id,
                        trace_region_scene_data([32, 64, 240]),
                    )]))
                    .build(),
            ))
    }

    fn frame_with_scene_prepare_runtime_trace_region_scene_data_and_mixed_legacy_payload(
    ) -> ViewportRenderFrame {
        let legacy_trace_region_id = 40;
        let runtime_only_trace_region_id = 41;
        let extract = extract_with_legacy_trace_region(legacy_trace_region_id);

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![
                    legacy_trace_region_id,
                    runtime_only_trace_region_id,
                    runtime_only_trace_region_id,
                ],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame::default()))
            .with_hybrid_gi_resolve_runtime(Some(
                HybridGiResolveRuntime::fixture()
                    .with_trace_region_scene_data(BTreeMap::from([
                        (
                            legacy_trace_region_id,
                            trace_region_scene_data([240, 96, 48]),
                        ),
                        (
                            runtime_only_trace_region_id,
                            trace_region_scene_data([32, 64, 240]),
                        ),
                    ]))
                    .build(),
            ))
    }

    fn frame_with_stripped_runtime_trace_region_scene_data_backed_by_legacy_payload(
    ) -> ViewportRenderFrame {
        let trace_region_id = 40;
        let extract = extract_with_legacy_trace_region(trace_region_id);

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![trace_region_id],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(runtime_scene_truth_with_trace_regions(
                BTreeMap::from([(trace_region_id, trace_region_scene_data([32, 64, 240]))]),
            )))
    }

    fn frame_with_stripped_runtime_trace_region_scene_data_and_mixed_legacy_payload(
    ) -> ViewportRenderFrame {
        let legacy_trace_region_id = 40;
        let runtime_only_trace_region_id = 41;
        let extract = extract_with_legacy_trace_region(legacy_trace_region_id);

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![
                    legacy_trace_region_id,
                    runtime_only_trace_region_id,
                    runtime_only_trace_region_id,
                ],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(runtime_scene_truth_with_trace_regions(
                BTreeMap::from([
                    (
                        legacy_trace_region_id,
                        trace_region_scene_data([240, 96, 48]),
                    ),
                    (
                        runtime_only_trace_region_id,
                        trace_region_scene_data([32, 64, 240]),
                    ),
                ]),
            )))
    }

    fn runtime_scene_truth_with_trace_regions(
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(trace_region_scene_data)
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                700,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.25, 0.45, 0.75], 0.5),
            )]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([700]))
            .build()
    }

    fn extract_with_legacy_trace_region(trace_region_id: u32) -> RenderFrameExtract {
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(trace_region_id),
                region_id: trace_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });
        extract
    }

    fn trace_region_scene_data(rt_lighting_rgb: [u8; 3]) -> HybridGiResolveTraceRegionSceneData {
        HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, rt_lighting_rgb)
    }

    fn frame_with_budgeted_scene_representation_and_legacy_scheduled_trace_region(
    ) -> ViewportRenderFrame {
        let trace_region_id = 40;
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            card_budget: 1,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: u64::from(trace_region_id),
                region_id: trace_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 0.8,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32)).with_hybrid_gi_prepare(Some(
            crate::graphics::types::HybridGiPrepareFrame {
                resident_probes: Vec::new(),
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![trace_region_id],
                evictable_probe_ids: Vec::new(),
            },
        ))
    }

    fn frame_with_budgeted_scene_representation_and_legacy_extract_lineage() -> ViewportRenderFrame
    {
        let child_probe_id = 300;
        let stale_parent_probe_id = 400;
        let child_probe = RenderHybridGiProbe {
            probe_id: child_probe_id,
            parent_probe_id: Some(stale_parent_probe_id),
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let stale_parent_probe = RenderHybridGiProbe {
            probe_id: stale_parent_probe_id,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 2,
            trace_budget: 1,
            card_budget: 1,
            probes: vec![child_probe, stale_parent_probe],
            ..Default::default()
        });

        ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
    }
}
