use std::collections::BTreeSet;

use crate::core::framework::render::RenderHybridGiExtract;
use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;

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
        .probe_scene_driven_hierarchy_irradiance_ids
        .iter()
        .any(|&probe_id| runtime_probe_has_irradiance_scene_truth(runtime, probe_id))
        || runtime
            .probe_scene_driven_hierarchy_rt_lighting_ids
            .iter()
            .any(|&probe_id| runtime_probe_has_rt_lighting_scene_truth(runtime, probe_id))
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
            || (runtime.probe_rt_lighting_rgb.contains_key(&probe_id)
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
        .map(|runtime| !runtime.probe_parent_probes.is_empty())
        .unwrap_or(false)
        || frame_has_runtime_scene_truth(frame)
}

pub(super) fn frame_has_scheduled_trace_region_payload(frame: &ViewportRenderFrame) -> bool {
    !scheduled_live_trace_region_ids(frame).is_empty()
}

pub(in super::super) fn scheduled_live_trace_region_ids(frame: &ViewportRenderFrame) -> Vec<u32> {
    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return Vec::new();
    };
    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return Vec::new();
    };

    let trace_region_ids = extract
        .trace_regions
        .iter()
        .map(|region| region.region_id)
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
    legacy_parent_probe_id: Option<u32>,
) -> Vec<(u32, usize)> {
    if let Some(chain) = parent_probe_chain(frame, probe_id) {
        if !chain.is_empty() || runtime_parent_topology_is_authoritative(frame) {
            return chain;
        }
    }

    legacy_parent_probe_id
        .map(|parent_probe_id| vec![(parent_probe_id, 1)])
        .unwrap_or_default()
}

fn parent_probe_chain(frame: &ViewportRenderFrame, probe_id: u32) -> Option<Vec<(u32, usize)>> {
    if runtime_parent_topology_is_authoritative(frame) {
        if let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() {
            return Some(parent_probe_chain_from_runtime(runtime, probe_id));
        }
    }

    if let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() {
        let extract_chain = parent_probe_chain_from_extract(extract, probe_id);
        if !extract_chain.is_empty() {
            return Some(extract_chain);
        }
    }
    frame.hybrid_gi_resolve_runtime.as_ref().map(|_| Vec::new())
}

fn parent_probe_chain_from_extract(
    extract: &RenderHybridGiExtract,
    probe_id: u32,
) -> Vec<(u32, usize)> {
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

fn parent_probe_chain_from_runtime(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);
    let mut depth = 0usize;

    while let Some(parent_probe_id) = runtime.probe_parent_probes.get(&current_probe_id).copied() {
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
    if runtime_parent_topology_is_authoritative(frame) {
        if let Some(runtime) = frame.hybrid_gi_resolve_runtime.as_ref() {
            return Some(descendant_probe_chain_from_runtime(runtime, probe_id));
        }
    }

    if let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() {
        let extract_chain = descendant_probe_chain_from_extract(extract, probe_id);
        if !extract_chain.is_empty() {
            return Some(extract_chain);
        }
    }
    frame.hybrid_gi_resolve_runtime.as_ref().map(|_| Vec::new())
}

fn descendant_probe_chain_from_extract(
    extract: &RenderHybridGiExtract,
    probe_id: u32,
) -> Vec<(u32, usize)> {
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

fn descendant_probe_chain_from_runtime(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Vec<(u32, usize)> {
    let mut chain = Vec::new();
    let mut stack = runtime
        .probe_parent_probes
        .iter()
        .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
            (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
        })
        .collect::<Vec<_>>();
    let mut visited_probe_ids = BTreeSet::new();

    while let Some((candidate_probe_id, depth)) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe_id) {
            continue;
        }

        chain.push((candidate_probe_id, depth));
        stack.extend(runtime.probe_parent_probes.iter().filter_map(
            |(&grandchild_probe_id, &parent_probe_id)| {
                (parent_probe_id == candidate_probe_id).then_some((grandchild_probe_id, depth + 1))
            },
        ));
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
