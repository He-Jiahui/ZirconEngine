use std::collections::BTreeSet;

use crate::core::framework::render::RenderHybridGiProbe;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

use super::hybrid_gi_hierarchy_rt_lighting::{
    current_rt_lighting_surface_cache_proxy_rgb_and_support,
    current_rt_lighting_surface_cache_proxy_rgb_support_and_quality,
};
use super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, frame_has_runtime_scene_truth,
    frame_has_scheduled_trace_region_payload, gather_runtime_descendant_chain_rgb,
    gather_runtime_descendant_chain_rgb_without_depth_falloff,
    gather_runtime_descendant_chain_support_and_quality_without_depth_falloff,
    gather_runtime_descendant_chain_support_and_revision_without_depth_falloff,
    gather_runtime_parent_chain_rgb, gather_runtime_parent_chain_rgb_without_depth_falloff,
    gather_runtime_parent_chain_support_and_quality_without_depth_falloff,
    gather_runtime_parent_chain_support_and_revision_without_depth_falloff,
    runtime_probe_lineage_has_scene_truth, runtime_resolve_weight_support,
    temporal_parent_probe_chain,
};
use super::scene_prepare_surface_cache_samples::{
    scene_prepare_surface_cache_fallback_rgb_and_support,
    scene_prepare_surface_cache_fallback_rgb_support_and_quality,
};

const TEMPORAL_SIGNATURE_BUCKETS: u32 = 255;
const PROBE_SIGNATURE_SEED: u32 = 0x9E37_79B9;
const PARENT_SIGNATURE_SEED: u32 = 0x85EB_CA77;
const NO_PARENT_SIGNATURE_SEED: u32 = 0xA5A5_5A5A;
const ANCESTOR_SIGNATURE_DEPTH_SEED: u32 = 0x27D4_EB2F;
const SURFACE_CACHE_SIGNATURE_SEED: u32 = 0x1656_67B1;
const SURFACE_CACHE_SUPPORT_SEED: u32 = 0xC2B2_AE35;
const RUNTIME_IRRADIANCE_SIGNATURE_SEED: u32 = 0x4B1D_93A7;
const RUNTIME_IRRADIANCE_SUPPORT_SEED: u32 = 0x91E1_0DA5;
const RUNTIME_IRRADIANCE_VALIDITY_SEED: u32 = 0x3F84_D5B5;
const RUNTIME_IRRADIANCE_REVISION_SEED: u32 = 0x5C71_2AF3;
const RUNTIME_RT_SIGNATURE_SEED: u32 = 0x6C8E_9CF5;
const RUNTIME_RT_SUPPORT_SEED: u32 = 0xD3A2_647B;
const RUNTIME_RT_VALIDITY_SEED: u32 = 0xA24B_7E19;
const RUNTIME_RT_REVISION_SEED: u32 = 0x7E36_41AD;
const SURFACE_CACHE_SIGNATURE_SUPPORT_NORMALIZER: f32 = 1.5;
const RUNTIME_SIGNATURE_SUPPORT_NORMALIZER: f32 = 0.75;
const SURFACE_CACHE_PROXY_SCENE_TRUTH_CONFIDENCE_SCALE: f32 = 0.85;
const RUNTIME_EXACT_SCENE_TRUTH_CONFIDENCE_SCALE: f32 = 1.0;
const RUNTIME_INHERITED_SCENE_TRUTH_CONFIDENCE_SCALE: f32 = 0.85;
const RUNTIME_DESCENDANT_SCENE_TRUTH_CONFIDENCE_SCALE: f32 = 0.7;
const RUNTIME_EXACT_SCENE_TRUTH_REVISION_SEED: u32 = 0xC1D3_94E7;
const RUNTIME_INHERITED_SCENE_TRUTH_REVISION_SEED: u32 = 0xE54A_61C3;
const RUNTIME_DESCENDANT_SCENE_TRUTH_REVISION_SEED: u32 = 0x91B7_2D59;
const SCENE_TRUTH_SIGNATURE_SEED: u32 = 0x6E27_8C41;

pub(super) fn hybrid_gi_temporal_signature(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    parent_probe_id: Option<u32>,
    source: Option<&RenderHybridGiProbe>,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> f32 {
    let runtime_irradiance_signature_source = runtime_scene_truth_signature_source(
        frame,
        probe_id,
        HybridGiResolveRuntime::hierarchy_irradiance,
        HybridGiResolveRuntime::hierarchy_irradiance_includes_scene_truth,
    );
    let runtime_rt_signature_source = runtime_scene_truth_signature_source(
        frame,
        probe_id,
        runtime_rt_lighting_temporal_source,
        HybridGiResolveRuntime::hierarchy_rt_lighting_includes_scene_truth,
    );
    let rt_continuation_runtime_source = source
        .and_then(|source| {
            frame
                .hybrid_gi_resolve_runtime
                .as_ref()
                .and_then(|runtime| runtime_rt_lighting_temporal_source(runtime, source.probe_id))
        })
        .filter(|source| source[3] > f32::EPSILON);
    let rt_continuation_surface_cache_signature_source = rt_continuation_runtime_source
        .is_some()
        .then(|| {
            source.and_then(|source| {
                current_rt_lighting_surface_cache_proxy_rgb_and_support(
                    frame,
                    source,
                    scene_prepare_resources,
                )
            })
        })
        .flatten();
    let surface_cache_signature_source = if runtime_rt_signature_source.is_some() {
        None
    } else if rt_continuation_surface_cache_signature_source.is_some() {
        rt_continuation_surface_cache_signature_source
    } else if runtime_irradiance_signature_source.is_some() {
        None
    } else {
        source.and_then(|source| {
            surface_cache_proxy_signature_source_in_current_irradiance(
                frame,
                source,
                scene_prepare_resources,
            )
        })
    };
    let has_scene_truth_signature_authority = surface_cache_signature_source.is_some()
        || runtime_irradiance_signature_source.is_some()
        || runtime_rt_signature_source.is_some();

    let mut mixed_signature = if has_scene_truth_signature_authority {
        SCENE_TRUTH_SIGNATURE_SEED
    } else {
        let parent_chain = temporal_parent_probe_chain(frame, probe_id, parent_probe_id);
        let parent_signature = parent_chain
            .first()
            .map(|(parent_probe_id, _)| *parent_probe_id)
            .unwrap_or(NO_PARENT_SIGNATURE_SEED);
        let mut lineage_signature = mix_signature_words(
            probe_id ^ PROBE_SIGNATURE_SEED,
            parent_signature ^ PARENT_SIGNATURE_SEED,
        );
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        for (ancestor_probe_id, depth) in parent_chain {
            if !visited_probe_ids.insert(ancestor_probe_id) {
                break;
            }
            let depth = depth as u32;
            lineage_signature = mix_signature_words(
                lineage_signature ^ ancestor_probe_id.rotate_left(depth % u32::BITS),
                ancestor_probe_id ^ ANCESTOR_SIGNATURE_DEPTH_SEED.wrapping_mul(depth),
            );
        }
        lineage_signature
    };
    if let Some((surface_cache_rgb, support)) = surface_cache_signature_source {
        mixed_signature = mix_signature_rgb_and_support(
            mixed_signature,
            surface_cache_rgb,
            support,
            SURFACE_CACHE_SIGNATURE_SUPPORT_NORMALIZER,
            SURFACE_CACHE_SIGNATURE_SEED,
            SURFACE_CACHE_SUPPORT_SEED,
        );
    }
    if let Some(runtime_irradiance) = runtime_irradiance_signature_source {
        mixed_signature = mix_signature_rgb_and_support(
            mixed_signature,
            [
                runtime_irradiance[0],
                runtime_irradiance[1],
                runtime_irradiance[2],
            ],
            runtime_irradiance[3],
            RUNTIME_SIGNATURE_SUPPORT_NORMALIZER,
            RUNTIME_IRRADIANCE_SIGNATURE_SEED,
            RUNTIME_IRRADIANCE_SUPPORT_SEED,
        );
        if let Some((quality, freshness)) = runtime_scene_truth_signature_validity(
            frame,
            probe_id,
            HybridGiResolveRuntime::hierarchy_irradiance,
            HybridGiResolveRuntime::hierarchy_irradiance_includes_scene_truth,
            HybridGiResolveRuntime::hierarchy_irradiance_scene_truth_quality,
            HybridGiResolveRuntime::hierarchy_irradiance_scene_truth_freshness,
        ) {
            mixed_signature = mix_signature_quality_and_freshness(
                mixed_signature,
                quality,
                freshness,
                RUNTIME_IRRADIANCE_VALIDITY_SEED,
            );
        }
        if let Some(revision_signature) = runtime_scene_truth_signature_revision(
            frame,
            probe_id,
            HybridGiResolveRuntime::hierarchy_irradiance,
            HybridGiResolveRuntime::hierarchy_irradiance_includes_scene_truth,
            HybridGiResolveRuntime::hierarchy_irradiance_scene_truth_revision,
        ) {
            mixed_signature = mix_signature_words(
                mixed_signature ^ RUNTIME_IRRADIANCE_REVISION_SEED,
                revision_signature.rotate_left(13),
            );
        }
    }
    if let Some(runtime_rt_lighting) = runtime_rt_signature_source {
        mixed_signature = mix_signature_rgb_and_support(
            mixed_signature,
            [
                runtime_rt_lighting[0],
                runtime_rt_lighting[1],
                runtime_rt_lighting[2],
            ],
            runtime_rt_lighting[3],
            RUNTIME_SIGNATURE_SUPPORT_NORMALIZER,
            RUNTIME_RT_SIGNATURE_SEED,
            RUNTIME_RT_SUPPORT_SEED,
        );
        if let Some((quality, freshness)) = runtime_scene_truth_signature_validity(
            frame,
            probe_id,
            runtime_rt_lighting_temporal_source,
            HybridGiResolveRuntime::hierarchy_rt_lighting_includes_scene_truth,
            HybridGiResolveRuntime::hierarchy_rt_lighting_scene_truth_quality,
            HybridGiResolveRuntime::hierarchy_rt_lighting_scene_truth_freshness,
        ) {
            mixed_signature = mix_signature_quality_and_freshness(
                mixed_signature,
                quality,
                freshness,
                RUNTIME_RT_VALIDITY_SEED,
            );
        }
        if let Some(revision_signature) = runtime_scene_truth_signature_revision(
            frame,
            probe_id,
            runtime_rt_lighting_temporal_source,
            HybridGiResolveRuntime::hierarchy_rt_lighting_includes_scene_truth,
            HybridGiResolveRuntime::hierarchy_rt_lighting_scene_truth_revision,
        ) {
            mixed_signature = mix_signature_words(
                mixed_signature ^ RUNTIME_RT_REVISION_SEED,
                revision_signature.rotate_left(13),
            );
        }
    }
    let signature_bucket = 1 + (mixed_signature % TEMPORAL_SIGNATURE_BUCKETS);
    signature_bucket as f32 / TEMPORAL_SIGNATURE_BUCKETS as f32
}

pub(super) fn hybrid_gi_temporal_scene_truth_confidence(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source: Option<&RenderHybridGiProbe>,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> f32 {
    let runtime_irradiance_confidence = runtime_scene_truth_confidence(
        frame,
        probe_id,
        HybridGiResolveRuntime::hierarchy_irradiance,
        HybridGiResolveRuntime::hierarchy_irradiance_includes_scene_truth,
        HybridGiResolveRuntime::hierarchy_irradiance_scene_truth_quality,
        HybridGiResolveRuntime::hierarchy_irradiance_scene_truth_freshness,
    );
    let runtime_rt_lighting_confidence = runtime_scene_truth_confidence(
        frame,
        probe_id,
        runtime_rt_lighting_temporal_source,
        HybridGiResolveRuntime::hierarchy_rt_lighting_includes_scene_truth,
        HybridGiResolveRuntime::hierarchy_rt_lighting_scene_truth_quality,
        HybridGiResolveRuntime::hierarchy_rt_lighting_scene_truth_freshness,
    );
    let runtime_irradiance_signature_source = runtime_scene_truth_signature_source(
        frame,
        probe_id,
        HybridGiResolveRuntime::hierarchy_irradiance,
        HybridGiResolveRuntime::hierarchy_irradiance_includes_scene_truth,
    );
    let runtime_rt_signature_source = runtime_scene_truth_signature_source(
        frame,
        probe_id,
        runtime_rt_lighting_temporal_source,
        HybridGiResolveRuntime::hierarchy_rt_lighting_includes_scene_truth,
    );
    let rt_continuation_runtime_source = source
        .and_then(|source| {
            frame
                .hybrid_gi_resolve_runtime
                .as_ref()
                .and_then(|runtime| runtime_rt_lighting_temporal_source(runtime, source.probe_id))
        })
        .filter(|source| source[3] > f32::EPSILON);
    let rt_continuation_surface_cache_confidence = rt_continuation_runtime_source
        .is_some()
        .then(|| {
            source
                .and_then(|source| {
                    current_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
                        frame,
                        source,
                        scene_prepare_resources,
                    )
                })
                .map(|(_, support, confidence_quality)| {
                    (support / SURFACE_CACHE_SIGNATURE_SUPPORT_NORMALIZER.max(f32::EPSILON))
                        .clamp(0.0, 1.0)
                        * confidence_quality.clamp(0.0, 1.0)
                        * SURFACE_CACHE_PROXY_SCENE_TRUTH_CONFIDENCE_SCALE.clamp(0.0, 1.0)
                })
        })
        .flatten();
    let runtime_lineage_scene_truth = runtime_probe_lineage_has_scene_truth(frame, probe_id)
        || (frame.hybrid_gi_scene_prepare.is_some() && frame_has_runtime_scene_truth(frame));
    let surface_cache_confidence = if runtime_rt_signature_source.is_some()
        || runtime_rt_lighting_confidence > f32::EPSILON
        || runtime_lineage_scene_truth
    {
        0.0
    } else if let Some(confidence) = rt_continuation_surface_cache_confidence {
        confidence
    } else if runtime_irradiance_signature_source.is_some()
        || runtime_irradiance_confidence > f32::EPSILON
    {
        0.0
    } else {
        source
            .map(|source| {
                surface_cache_proxy_confidence_in_current_gi(frame, source, scene_prepare_resources)
            })
            .unwrap_or(0.0)
    };

    combine_temporal_confidences(&[
        surface_cache_confidence,
        runtime_irradiance_confidence,
        runtime_rt_lighting_confidence,
    ])
}

fn surface_cache_proxy_confidence_in_current_gi(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> f32 {
    if let Some((_, support, confidence_quality)) =
        current_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
            frame,
            source,
            scene_prepare_resources,
        )
    {
        return (support / SURFACE_CACHE_SIGNATURE_SUPPORT_NORMALIZER.max(f32::EPSILON))
            .clamp(0.0, 1.0)
            * confidence_quality.clamp(0.0, 1.0)
            * SURFACE_CACHE_PROXY_SCENE_TRUTH_CONFIDENCE_SCALE.clamp(0.0, 1.0);
    }

    frame
        .hybrid_gi_scene_prepare
        .as_ref()
        .and_then(|scene_prepare| {
            surface_cache_proxy_participates_in_current_irradiance(frame, source.probe_id).then(
                || {
                    scene_prepare_surface_cache_fallback_rgb_support_and_quality(
                        scene_prepare,
                        source.position,
                        source.radius,
                        scene_prepare_resources,
                    )
                },
            )
        })
        .flatten()
        .map(|(_, support, confidence_quality)| {
            (support / SURFACE_CACHE_SIGNATURE_SUPPORT_NORMALIZER.max(f32::EPSILON)).clamp(0.0, 1.0)
                * confidence_quality.clamp(0.0, 1.0)
                * SURFACE_CACHE_PROXY_SCENE_TRUTH_CONFIDENCE_SCALE.clamp(0.0, 1.0)
        })
        .unwrap_or(0.0)
}

fn surface_cache_proxy_signature_source_in_current_irradiance(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32)> {
    surface_cache_proxy_participates_in_current_irradiance(frame, source.probe_id)
        .then(|| {
            frame
                .hybrid_gi_scene_prepare
                .as_ref()
                .and_then(|scene_prepare| {
                    scene_prepare_surface_cache_fallback_rgb_and_support(
                        scene_prepare,
                        source.position,
                        source.radius,
                        scene_prepare_resources,
                    )
                })
        })
        .flatten()
}

fn surface_cache_proxy_participates_in_current_irradiance(
    frame: &ViewportRenderFrame,
    probe_id: u32,
) -> bool {
    let scene_driven_frame = frame.hybrid_gi_scene_prepare.is_some();
    let current_trace_schedule_is_empty = !frame_has_scheduled_trace_region_payload(frame);
    let exact_runtime_irradiance = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_irradiance(probe_id))
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_irradiance_includes_scene_truth(probe_id))
        .unwrap_or(false);
    let exact_scene_truth_runtime_irradiance =
        exact_runtime_irradiance.filter(|_| exact_runtime_includes_scene_truth);
    let exact_continuation_runtime_irradiance =
        exact_runtime_irradiance.filter(|_| !exact_runtime_includes_scene_truth);
    let exact_scene_truth_runtime_irradiance_present =
        exact_scene_truth_runtime_irradiance.is_some();
    let inherited_scene_truth_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_parent_chain_rgb_without_depth_falloff(
                frame,
                probe_id,
                |runtime, ancestor_probe_id| {
                    runtime
                        .hierarchy_irradiance_includes_scene_truth(ancestor_probe_id)
                        .then(|| runtime.hierarchy_irradiance(ancestor_probe_id))
                        .flatten()
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
                },
            )
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let inherited_continuation_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_parent_chain_rgb(frame, probe_id, |runtime, ancestor_probe_id| {
                if runtime.hierarchy_irradiance_includes_scene_truth(ancestor_probe_id) {
                    return None;
                }

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
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_descendant_chain_rgb_without_depth_falloff(
                frame,
                probe_id,
                |runtime, descendant_probe_id| {
                    if !runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                        return None;
                    }

                    runtime
                        .hierarchy_irradiance(descendant_probe_id)
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
                },
            )
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_continuation_runtime_irradiance =
        (!exact_scene_truth_runtime_irradiance_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb(
                    frame,
                    probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime.hierarchy_irradiance(descendant_probe_id).map(
                            |hierarchy_irradiance| {
                                (
                                    [
                                        hierarchy_irradiance[0],
                                        hierarchy_irradiance[1],
                                        hierarchy_irradiance[2],
                                    ],
                                    hierarchy_irradiance[3],
                                )
                            },
                        )
                    },
                )
            })
            .flatten()
            .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let scene_truth_runtime_irradiance = blend_runtime_rgb_lineage_sources(
        exact_scene_truth_runtime_irradiance,
        inherited_scene_truth_runtime_irradiance,
        descendant_scene_truth_runtime_irradiance,
    );
    let continuation_runtime_irradiance = blend_runtime_rgb_lineage_sources(
        exact_continuation_runtime_irradiance,
        inherited_continuation_runtime_irradiance,
        descendant_continuation_runtime_irradiance,
    );
    let selected_runtime_irradiance_is_scene_truth =
        scene_driven_frame && scene_truth_runtime_irradiance.is_some();
    let selected_runtime_irradiance = if selected_runtime_irradiance_is_scene_truth {
        scene_truth_runtime_irradiance
    } else {
        blend_runtime_rgb_lineage_sources(
            scene_truth_runtime_irradiance,
            continuation_runtime_irradiance,
            None,
        )
    };

    if selected_runtime_irradiance.is_some() {
        return current_trace_schedule_is_empty && !selected_runtime_irradiance_is_scene_truth;
    }

    true
}

fn combine_temporal_confidences(confidences: &[f32]) -> f32 {
    let inverse_confidence = confidences
        .iter()
        .fold(1.0_f32, |inverse_confidence, confidence| {
            inverse_confidence * (1.0 - confidence.clamp(0.0, 1.0))
        });
    (1.0 - inverse_confidence).clamp(0.0, 1.0)
}

fn runtime_scene_truth_confidence<S, T, Q, F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_probe: S,
    includes_scene_truth: T,
    quality_for_probe: Q,
    freshness_for_probe: F,
) -> f32
where
    S: Fn(&HybridGiResolveRuntime, u32) -> Option<[f32; 4]> + Copy,
    T: Fn(&HybridGiResolveRuntime, u32) -> bool + Copy,
    Q: Fn(&HybridGiResolveRuntime, u32) -> f32 + Copy,
    F: Fn(&HybridGiResolveRuntime, u32) -> f32 + Copy,
{
    let exact_confidence = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .filter(|runtime| includes_scene_truth(runtime, probe_id))
        .and_then(|runtime| {
            source_for_probe(runtime, probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| {
                    (
                        source[3],
                        quality_for_probe(runtime, probe_id),
                        freshness_for_probe(runtime, probe_id),
                    )
                })
        })
        .map(|(support, source_quality, source_freshness)| {
            normalized_runtime_scene_truth_confidence(
                support,
                RUNTIME_EXACT_SCENE_TRUTH_CONFIDENCE_SCALE,
                source_quality,
                source_freshness,
            )
        })
        .unwrap_or(0.0);
    if exact_confidence > 0.0 {
        return exact_confidence;
    }

    let inherited_confidence =
        gather_runtime_parent_chain_support_and_quality_without_depth_falloff(
            frame,
            probe_id,
            |runtime, ancestor_probe_id| {
                if !includes_scene_truth(runtime, ancestor_probe_id) {
                    return None;
                }

                source_for_probe(runtime, ancestor_probe_id)
                    .filter(|source| source[3] > f32::EPSILON)
                    .map(|source| {
                        (
                            source[3],
                            quality_for_probe(runtime, ancestor_probe_id),
                            freshness_for_probe(runtime, ancestor_probe_id),
                        )
                    })
            },
        )
        .map(|(support, source_quality, source_freshness)| {
            normalized_runtime_scene_truth_confidence(
                support,
                RUNTIME_INHERITED_SCENE_TRUTH_CONFIDENCE_SCALE,
                source_quality,
                source_freshness,
            )
        })
        .unwrap_or(0.0);
    let descendant_confidence =
        gather_runtime_descendant_chain_support_and_quality_without_depth_falloff(
            frame,
            probe_id,
            |runtime, descendant_probe_id| {
                if !includes_scene_truth(runtime, descendant_probe_id) {
                    return None;
                }

                source_for_probe(runtime, descendant_probe_id)
                    .filter(|source| source[3] > f32::EPSILON)
                    .map(|source| {
                        (
                            source[3],
                            quality_for_probe(runtime, descendant_probe_id),
                            freshness_for_probe(runtime, descendant_probe_id),
                        )
                    })
            },
        )
        .map(|(support, source_quality, source_freshness)| {
            normalized_runtime_scene_truth_confidence(
                support,
                RUNTIME_DESCENDANT_SCENE_TRUTH_CONFIDENCE_SCALE,
                source_quality,
                source_freshness,
            )
        })
        .unwrap_or(0.0);

    combine_temporal_confidences(&[
        exact_confidence,
        inherited_confidence,
        descendant_confidence,
    ])
}

fn normalized_runtime_scene_truth_confidence(
    support: f32,
    lineage_scale: f32,
    source_quality: f32,
    source_freshness: f32,
) -> f32 {
    (support / RUNTIME_SIGNATURE_SUPPORT_NORMALIZER.max(f32::EPSILON)).clamp(0.0, 1.0)
        * lineage_scale.clamp(0.0, 1.0)
        * source_quality.clamp(0.0, 1.0)
        * source_freshness.clamp(0.0, 1.0)
}

fn runtime_rt_lighting_temporal_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<[f32; 4]> {
    if let Some(hierarchy_rt_lighting) = runtime
        .hierarchy_rt_lighting(probe_id)
        .filter(|source| source[3] > f32::EPSILON)
    {
        return Some(hierarchy_rt_lighting);
    }

    runtime
        .probe_rt_lighting_rgb
        .get(&probe_id)
        .copied()
        .and_then(|rgb| {
            let support =
                runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(probe_id));
            (support > f32::EPSILON).then_some([
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                support,
            ])
        })
}

fn runtime_scene_truth_signature_source<S, T>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_probe: S,
    includes_scene_truth: T,
) -> Option<[f32; 4]>
where
    S: Fn(&HybridGiResolveRuntime, u32) -> Option<[f32; 4]> + Copy,
    T: Fn(&HybridGiResolveRuntime, u32) -> bool + Copy,
{
    let exact = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .filter(|runtime| includes_scene_truth(runtime, probe_id))
        .and_then(|runtime| source_for_probe(runtime, probe_id))
        .filter(|source| source[3] > f32::EPSILON);
    if exact.is_some() {
        return exact;
    }

    let inherited = gather_runtime_parent_chain_rgb_without_depth_falloff(
        frame,
        probe_id,
        |runtime, ancestor_probe_id| {
            if !includes_scene_truth(runtime, ancestor_probe_id) {
                return None;
            }

            source_for_probe(runtime, ancestor_probe_id)
                .map(|source| ([source[0], source[1], source[2]], source[3]))
        },
    )
    .filter(|source| source[3] > f32::EPSILON);
    let descendant = gather_runtime_descendant_chain_rgb_without_depth_falloff(
        frame,
        probe_id,
        |runtime, descendant_probe_id| {
            if !includes_scene_truth(runtime, descendant_probe_id) {
                return None;
            }

            source_for_probe(runtime, descendant_probe_id)
                .map(|source| ([source[0], source[1], source[2]], source[3]))
        },
    )
    .filter(|source| source[3] > f32::EPSILON);

    blend_runtime_rgb_lineage_sources(exact, inherited, descendant)
}

fn runtime_scene_truth_signature_validity<S, T, Q, F>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_probe: S,
    includes_scene_truth: T,
    quality_for_probe: Q,
    freshness_for_probe: F,
) -> Option<(f32, f32)>
where
    S: Fn(&HybridGiResolveRuntime, u32) -> Option<[f32; 4]> + Copy,
    T: Fn(&HybridGiResolveRuntime, u32) -> bool + Copy,
    Q: Fn(&HybridGiResolveRuntime, u32) -> f32 + Copy,
    F: Fn(&HybridGiResolveRuntime, u32) -> f32 + Copy,
{
    let exact = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .filter(|runtime| includes_scene_truth(runtime, probe_id))
        .and_then(|runtime| {
            source_for_probe(runtime, probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| {
                    (
                        source[3],
                        quality_for_probe(runtime, probe_id),
                        freshness_for_probe(runtime, probe_id),
                    )
                })
        });
    if let Some((_, quality, freshness)) = exact {
        return Some((quality.clamp(0.0, 1.0), freshness.clamp(0.0, 1.0)));
    }

    let inherited = gather_runtime_parent_chain_support_and_quality_without_depth_falloff(
        frame,
        probe_id,
        |runtime, ancestor_probe_id| {
            if !includes_scene_truth(runtime, ancestor_probe_id) {
                return None;
            }

            source_for_probe(runtime, ancestor_probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| {
                    (
                        source[3],
                        quality_for_probe(runtime, ancestor_probe_id),
                        freshness_for_probe(runtime, ancestor_probe_id),
                    )
                })
        },
    );
    let descendant = gather_runtime_descendant_chain_support_and_quality_without_depth_falloff(
        frame,
        probe_id,
        |runtime, descendant_probe_id| {
            if !includes_scene_truth(runtime, descendant_probe_id) {
                return None;
            }

            source_for_probe(runtime, descendant_probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| {
                    (
                        source[3],
                        quality_for_probe(runtime, descendant_probe_id),
                        freshness_for_probe(runtime, descendant_probe_id),
                    )
                })
        },
    );

    blend_runtime_quality_and_freshness_sources(exact, inherited, descendant)
}

fn runtime_scene_truth_signature_revision<S, T, R>(
    frame: &ViewportRenderFrame,
    probe_id: u32,
    source_for_probe: S,
    includes_scene_truth: T,
    revision_for_probe: R,
) -> Option<u32>
where
    S: Fn(&HybridGiResolveRuntime, u32) -> Option<[f32; 4]> + Copy,
    T: Fn(&HybridGiResolveRuntime, u32) -> bool + Copy,
    R: Fn(&HybridGiResolveRuntime, u32) -> u32 + Copy,
{
    let exact = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .filter(|runtime| includes_scene_truth(runtime, probe_id))
        .and_then(|runtime| {
            source_for_probe(runtime, probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| (source[3], revision_for_probe(runtime, probe_id)))
        });
    if exact.is_some() {
        return mix_runtime_scene_truth_revisions(exact, None, None);
    }

    let inherited = gather_runtime_parent_chain_support_and_revision_without_depth_falloff(
        frame,
        probe_id,
        |runtime, ancestor_probe_id| {
            if !includes_scene_truth(runtime, ancestor_probe_id) {
                return None;
            }

            source_for_probe(runtime, ancestor_probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| (source[3], revision_for_probe(runtime, ancestor_probe_id)))
        },
    );
    let descendant = gather_runtime_descendant_chain_support_and_revision_without_depth_falloff(
        frame,
        probe_id,
        |runtime, descendant_probe_id| {
            if !includes_scene_truth(runtime, descendant_probe_id) {
                return None;
            }

            source_for_probe(runtime, descendant_probe_id)
                .filter(|source| source[3] > f32::EPSILON)
                .map(|source| (source[3], revision_for_probe(runtime, descendant_probe_id)))
        },
    );

    mix_runtime_scene_truth_revisions(exact, inherited, descendant)
}

fn blend_runtime_quality_and_freshness_sources(
    exact: Option<(f32, f32, f32)>,
    inherited: Option<(f32, f32, f32)>,
    descendant: Option<(f32, f32, f32)>,
) -> Option<(f32, f32)> {
    let mut weighted_quality = 0.0_f32;
    let mut weighted_freshness = 0.0_f32;
    let mut total_support = 0.0_f32;
    for (support, quality, freshness) in [exact, inherited, descendant].into_iter().flatten() {
        if support <= f32::EPSILON {
            continue;
        }

        weighted_quality += quality.clamp(0.0, 1.0) * support;
        weighted_freshness += freshness.clamp(0.0, 1.0) * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((
        (weighted_quality / total_support).clamp(0.0, 1.0),
        (weighted_freshness / total_support).clamp(0.0, 1.0),
    ))
}

fn mix_runtime_scene_truth_revisions(
    exact: Option<(f32, u32)>,
    inherited: Option<(f32, u32)>,
    descendant: Option<(f32, u32)>,
) -> Option<u32> {
    let mut mixed_revision = 0u32;
    let mut has_revision = false;
    for (lineage_seed, source) in [
        (RUNTIME_EXACT_SCENE_TRUTH_REVISION_SEED, exact),
        (RUNTIME_INHERITED_SCENE_TRUTH_REVISION_SEED, inherited),
        (RUNTIME_DESCENDANT_SCENE_TRUTH_REVISION_SEED, descendant),
    ] {
        let Some((support, revision)) = source else {
            continue;
        };
        if support <= f32::EPSILON {
            continue;
        }

        let support_q = quantize_signature_channel(
            (support / RUNTIME_SIGNATURE_SUPPORT_NORMALIZER.max(f32::EPSILON)).clamp(0.0, 1.0),
        );
        let packed_revision = revision ^ support_q.rotate_left(8);
        mixed_revision = if has_revision {
            mix_signature_words(
                mixed_revision ^ lineage_seed,
                packed_revision.rotate_left(5),
            )
        } else {
            mix_signature_words(lineage_seed, packed_revision.rotate_left(5))
        };
        has_revision = true;
    }

    has_revision.then_some(mixed_revision)
}

fn mix_signature_rgb_and_support(
    mixed_signature: u32,
    rgb: [f32; 3],
    support: f32,
    support_normalizer: f32,
    signature_seed: u32,
    support_seed: u32,
) -> u32 {
    let rgb_q = [
        quantize_signature_channel(rgb[0]),
        quantize_signature_channel(rgb[1]),
        quantize_signature_channel(rgb[2]),
    ];
    let support_q = quantize_signature_channel(
        (support / support_normalizer.max(f32::EPSILON)).clamp(0.0, 1.0),
    );
    let packed_signature = rgb_q[0] | (rgb_q[1] << 8) | (rgb_q[2] << 16) | (support_q << 24);
    let mixed_signature = mix_signature_words(mixed_signature ^ signature_seed, packed_signature);
    mix_signature_words(
        mixed_signature ^ support_q.rotate_left(11),
        support_seed ^ packed_signature.rotate_left(7),
    )
}

fn mix_signature_quality_and_freshness(
    mixed_signature: u32,
    quality: f32,
    freshness: f32,
    validity_seed: u32,
) -> u32 {
    let quality_q = quantize_signature_channel(quality);
    let freshness_q = quantize_signature_channel(freshness);
    let packed_validity = quality_q | (freshness_q << 8);
    let mixed_signature = mix_signature_words(
        mixed_signature ^ validity_seed,
        packed_validity.rotate_left(9),
    );
    mix_signature_words(
        mixed_signature ^ freshness_q.rotate_left(17),
        validity_seed ^ packed_validity.rotate_left(21),
    )
}

fn mix_signature_words(left: u32, right: u32) -> u32 {
    let mut mixed = left.wrapping_add(0x7FEB_352D).wrapping_mul(0x846C_A68B);
    mixed ^= right.rotate_left(16);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x2C1B_3C6D);
    mixed ^ (mixed >> 12)
}

fn quantize_signature_channel(value: f32) -> u32 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u32
}
