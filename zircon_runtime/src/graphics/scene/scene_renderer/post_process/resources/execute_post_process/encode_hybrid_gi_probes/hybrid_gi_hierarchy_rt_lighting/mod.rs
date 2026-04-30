use std::collections::BTreeMap;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::hybrid_gi_trace_region_source::fallback_trace_region_sources_by_id;
use super::hybrid_gi_probe_source::{fallback_probe_sources_by_id, HybridGiProbeSource};
use super::runtime_parent_chain::{
    runtime_probe_lineage_has_scene_truth, runtime_rt_lighting_lineage_has_scene_truth,
    scheduled_live_trace_region_ids,
};
mod runtime_rt_sources;
pub(super) mod scene_prepare_rt_fallback;
mod scene_prepare_voxel_samples;
mod trace_region_inheritance;

use runtime_rt_sources::runtime_rt_lighting_selection;
use scene_prepare_rt_fallback::scene_prepare_voxel_fallback_rt_lighting;
use trace_region_inheritance::trace_region_inheritance_rt_lighting;

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn hybrid_gi_hierarchy_rt_lighting<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
) -> [f32; 4] {
    hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(frame, source, None)
}

pub(crate) fn hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources<
    S: HybridGiProbeSource + ?Sized,
>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 4] {
    let source_probe_id = source.probe_id();
    let has_scene_prepare = frame.hybrid_gi_scene_prepare.is_some();
    let runtime_probe_scene_truth = runtime_probe_lineage_has_scene_truth(frame, source_probe_id);
    let stripped_runtime_probe_scene_truth = !has_scene_prepare && runtime_probe_scene_truth;
    let scene_driven_frame = has_scene_prepare
        || stripped_runtime_probe_scene_truth
        || runtime_rt_lighting_lineage_has_scene_truth(frame, source_probe_id);
    let prepare = frame.hybrid_gi_prepare.as_ref();
    let scheduled_trace_region_ids = scheduled_live_trace_region_ids(frame);
    let probes_by_id =
        fallback_probe_sources_by_id(frame.extract.lighting.hybrid_global_illumination.as_ref());
    let trace_regions_by_id = fallback_trace_region_sources_by_id(
        frame.extract.lighting.hybrid_global_illumination.as_ref(),
    );
    let resident_prepare_by_id = prepare
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let runtime_selection = runtime_rt_lighting_selection(
        frame,
        source,
        &resident_prepare_by_id,
        scene_driven_frame,
        runtime_probe_scene_truth,
    );
    if let Some(runtime_rt_lighting) = runtime_selection.selected {
        if scene_driven_frame && !runtime_selection.selected_is_scene_truth {
            if let Some(scene_prepare_rt_lighting) =
                scene_prepare_voxel_fallback_rt_lighting(frame, source, scene_prepare_resources)
                    .filter(|scene_prepare_rt_lighting| scene_prepare_rt_lighting[3] > f32::EPSILON)
            {
                let total_support = runtime_rt_lighting[3] + scene_prepare_rt_lighting[3];
                if total_support > f32::EPSILON {
                    return [
                        (runtime_rt_lighting[0] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[0] * scene_prepare_rt_lighting[3])
                            / total_support,
                        (runtime_rt_lighting[1] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[1] * scene_prepare_rt_lighting[3])
                            / total_support,
                        (runtime_rt_lighting[2] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[2] * scene_prepare_rt_lighting[3])
                            / total_support,
                        total_support.clamp(0.0, 0.75),
                    ];
                }
            }
        }
        return runtime_rt_lighting;
    }
    let scene_prepare_voxel_fallback =
        scene_prepare_voxel_fallback_rt_lighting(frame, source, scene_prepare_resources);
    if scene_driven_frame {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    trace_region_inheritance_rt_lighting(
        frame,
        source,
        scene_prepare_voxel_fallback,
        &scheduled_trace_region_ids,
        &probes_by_id,
        &trace_regions_by_id,
        &resident_prepare_by_id,
    )
}

#[cfg(test)]
mod tests;
