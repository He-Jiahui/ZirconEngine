use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::runtime_parent_chain::{
    frame_has_scheduled_trace_region_source, runtime_irradiance_lineage_has_scene_truth,
    runtime_probe_lineage_has_scene_truth,
};

mod ancestor_prepare_inheritance;
mod runtime_irradiance_sources;
mod scene_prepare_irradiance_fallback;
#[cfg(test)]
mod tests;

use ancestor_prepare_inheritance::ancestor_prepare_irradiance_fallback;
use runtime_irradiance_sources::runtime_irradiance_selection;
use scene_prepare_irradiance_fallback::scene_prepare_surface_cache_irradiance_fallback;

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn hybrid_gi_hierarchy_irradiance<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
) -> [f32; 4] {
    hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources(frame, source, None)
}

pub(crate) fn hybrid_gi_hierarchy_irradiance_with_scene_prepare_resources<
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
        || runtime_irradiance_lineage_has_scene_truth(frame, source_probe_id);
    let current_trace_schedule_is_empty = !frame_has_scheduled_trace_region_source(frame);
    let runtime_selection =
        runtime_irradiance_selection(frame, source, scene_driven_frame, runtime_probe_scene_truth);
    if let Some(runtime_irradiance) = runtime_selection.selected {
        if current_trace_schedule_is_empty && !runtime_selection.selected_is_scene_truth {
            if let Some(scene_prepare_irradiance) = scene_prepare_surface_cache_irradiance_fallback(
                frame,
                source,
                scene_prepare_resources,
            )
            .filter(|scene_prepare_irradiance| scene_prepare_irradiance[3] > f32::EPSILON)
            {
                let total_support = runtime_irradiance[3] + scene_prepare_irradiance[3];
                if total_support > f32::EPSILON {
                    return [
                        (runtime_irradiance[0] * runtime_irradiance[3]
                            + scene_prepare_irradiance[0] * scene_prepare_irradiance[3])
                            / total_support,
                        (runtime_irradiance[1] * runtime_irradiance[3]
                            + scene_prepare_irradiance[1] * scene_prepare_irradiance[3])
                            / total_support,
                        (runtime_irradiance[2] * runtime_irradiance[3]
                            + scene_prepare_irradiance[2] * scene_prepare_irradiance[3])
                            / total_support,
                        total_support.clamp(0.0, 0.75),
                    ];
                }
            }
        }
        return runtime_irradiance;
    }

    ancestor_prepare_irradiance_fallback(frame, source, scene_prepare_resources)
}
