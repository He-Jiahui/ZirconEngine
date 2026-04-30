use crate::graphics::types::{HybridGiResolveRuntime, ViewportRenderFrame};

use super::super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_descendant_chain_rgb_without_depth_falloff, gather_runtime_parent_chain_rgb,
    gather_runtime_parent_chain_rgb_without_depth_falloff,
};

pub(super) struct RuntimeIrradianceSelection {
    pub(super) selected: Option<[f32; 4]>,
    pub(super) selected_is_scene_truth: bool,
}

pub(super) fn runtime_irradiance_selection<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_driven_frame: bool,
    runtime_probe_scene_truth: bool,
) -> RuntimeIrradianceSelection {
    let source_probe_id = source.probe_id();
    let exact_runtime_irradiance = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .and_then(|runtime| runtime.hierarchy_irradiance(source_probe_id))
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_irradiance_includes_scene_truth(source_probe_id))
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
                source_probe_id,
                |runtime, ancestor_probe_id| {
                    if !runtime.hierarchy_irradiance_includes_scene_truth(ancestor_probe_id) {
                        return None;
                    }

                    runtime_irradiance_lineage_source(runtime, ancestor_probe_id)
                },
            )
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let inherited_continuation_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_parent_chain_rgb(frame, source_probe_id, |runtime, ancestor_probe_id| {
                if runtime.hierarchy_irradiance_includes_scene_truth(ancestor_probe_id) {
                    return None;
                }

                runtime_irradiance_lineage_source(runtime, ancestor_probe_id)
            })
        })
        .flatten()
        .filter(|runtime_irradiance| runtime_irradiance[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_irradiance = (!exact_scene_truth_runtime_irradiance_present)
        .then(|| {
            gather_runtime_descendant_chain_rgb_without_depth_falloff(
                frame,
                source_probe_id,
                |runtime, descendant_probe_id| {
                    if !runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                        return None;
                    }

                    runtime_irradiance_lineage_source(runtime, descendant_probe_id)
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
                    source_probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_irradiance_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime_irradiance_lineage_source(runtime, descendant_probe_id)
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
    let selected_is_scene_truth = scene_driven_frame && scene_truth_runtime_irradiance.is_some();
    let selected = if selected_is_scene_truth {
        scene_truth_runtime_irradiance
    } else if runtime_probe_scene_truth && scene_truth_runtime_irradiance.is_none() {
        None
    } else {
        blend_runtime_rgb_lineage_sources(
            scene_truth_runtime_irradiance,
            continuation_runtime_irradiance,
            None,
        )
    };

    RuntimeIrradianceSelection {
        selected,
        selected_is_scene_truth,
    }
}

fn runtime_irradiance_lineage_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<([f32; 3], f32)> {
    runtime
        .hierarchy_irradiance(probe_id)
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
}
