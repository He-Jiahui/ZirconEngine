use std::collections::BTreeMap;

use crate::graphics::types::{HybridGiPrepareProbe, HybridGiResolveRuntime, ViewportRenderFrame};

use super::super::super::hybrid_gi_trace_region_source::HybridGiTraceRegionSource;
use super::super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_descendant_chain_rgb_without_depth_falloff, gather_runtime_parent_chain_rgb,
    gather_runtime_parent_chain_rgb_without_depth_falloff, runtime_resolve_weight_support,
};

pub(super) struct RuntimeRtLightingSelection {
    pub(super) selected: Option<[f32; 4]>,
    pub(super) selected_is_scene_truth: bool,
}

pub(super) fn runtime_rt_lighting_selection<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
    resident_prepare_by_id: &BTreeMap<u32, &HybridGiPrepareProbe>,
    scene_driven_frame: bool,
    runtime_probe_scene_truth: bool,
) -> RuntimeRtLightingSelection {
    let source_probe_id = source.probe_id();
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_rt_lighting_includes_scene_truth(source_probe_id))
        .unwrap_or(false);
    let exact_runtime_rt_lighting = if exact_runtime_includes_scene_truth {
        frame
            .hybrid_gi_resolve_runtime
            .as_ref()
            .and_then(|runtime| {
                runtime_rt_lighting_packed_or_legacy_source(runtime, source_probe_id)
            })
    } else {
        runtime_hierarchy_rt_lighting(frame, source, resident_prepare_by_id)
    }
    .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let exact_scene_truth_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| exact_runtime_includes_scene_truth);
    let exact_continuation_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| !exact_runtime_includes_scene_truth);
    let exact_scene_truth_runtime_rt_lighting_present =
        exact_scene_truth_runtime_rt_lighting.is_some();
    let inherited_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb_without_depth_falloff(
                    frame,
                    source_probe_id,
                    |runtime, ancestor_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let inherited_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb(
                    frame,
                    source_probe_id,
                    |runtime, ancestor_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb_without_depth_falloff(
                    frame,
                    source_probe_id,
                    |runtime, descendant_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id)
                        {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb(
                    frame,
                    source_probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let scene_truth_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_scene_truth_runtime_rt_lighting,
        inherited_scene_truth_runtime_rt_lighting,
        descendant_scene_truth_runtime_rt_lighting,
    );
    let continuation_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_continuation_runtime_rt_lighting,
        inherited_continuation_runtime_rt_lighting,
        descendant_continuation_runtime_rt_lighting,
    );
    let selected_is_scene_truth = scene_driven_frame && scene_truth_runtime_rt_lighting.is_some();
    let selected = if selected_is_scene_truth {
        scene_truth_runtime_rt_lighting
    } else if runtime_probe_scene_truth && scene_truth_runtime_rt_lighting.is_none() {
        None
    } else {
        blend_runtime_rgb_lineage_sources(
            scene_truth_runtime_rt_lighting,
            continuation_runtime_rt_lighting,
            None,
        )
    };

    RuntimeRtLightingSelection {
        selected,
        selected_is_scene_truth,
    }
}

pub(super) fn runtime_rt_lighting_lineage_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<([f32; 3], f32)> {
    runtime_rt_lighting_packed_or_legacy_source(runtime, probe_id)
        .map(|source| ([source[0], source[1], source[2]], source[3]))
}

pub(super) fn runtime_rt_lighting_packed_or_legacy_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<[f32; 4]> {
    if let Some(hierarchy_rt_lighting) = runtime
        .hierarchy_rt_lighting(probe_id)
        .filter(|hierarchy_rt_lighting| hierarchy_rt_lighting[3] > f32::EPSILON)
    {
        return Some(hierarchy_rt_lighting);
    }

    runtime.probe_rt_lighting_rgb(probe_id).and_then(|rgb| {
        let support = runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(probe_id));
        (support > f32::EPSILON).then_some([
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
            support,
        ])
    })
}

pub(super) fn runtime_hierarchy_rt_lighting<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
    resident_prepare_by_id: &BTreeMap<u32, &HybridGiPrepareProbe>,
) -> Option<[f32; 4]> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let source_probe_id = source.probe_id();
    let direct_rt_lighting_rgb = runtime.probe_rt_lighting_rgb(source_probe_id);
    let hierarchy_rt_lighting = runtime.hierarchy_rt_lighting(source_probe_id);
    if direct_rt_lighting_rgb.is_none() && hierarchy_rt_lighting.is_none() {
        return None;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if let Some(direct_rt_lighting_rgb) = direct_rt_lighting_rgb {
        let direct_support = resident_prepare_by_id
            .get(&source_probe_id)
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
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

pub(super) fn hierarchy_trace_region_support<
    P: HybridGiProbeSource + ?Sized,
    R: HybridGiTraceRegionSource + ?Sized,
>(
    probe: &P,
    region: &R,
) -> f32 {
    let reach = (probe.radius().max(0.05) + region.bounds_radius().max(0.05)).max(0.05);
    let distance = probe.position().distance(region.bounds_center());
    let falloff = (1.0 - distance / reach).max(0.0);
    let coverage_weight = (0.35 + region.screen_coverage().clamp(0.0, 1.0) * 0.65).clamp(0.35, 1.0);
    falloff * falloff * coverage_weight
}

pub(super) fn hybrid_gi_trace_region_rt_lighting<R: HybridGiTraceRegionSource + ?Sized>(
    region: &R,
) -> [f32; 4] {
    let rt_lighting_rgb = region.rt_lighting_rgb();
    let rgb = [
        rt_lighting_rgb[0] as f32 / 255.0,
        rt_lighting_rgb[1] as f32 / 255.0,
        rt_lighting_rgb[2] as f32 / 255.0,
    ];
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]);

    [rgb[0], rgb[1], rgb[2], max_component]
}
