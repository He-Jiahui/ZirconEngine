use crate::graphics::runtime::hybrid_gi::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimeScenePrepareResources,
};
use crate::graphics::runtime::virtual_geometry::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeFeedback,
};
use crate::SceneRenderer;

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;

pub(super) fn collect_runtime_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
) -> RuntimeFeedbackBatch {
    RuntimeFeedbackBatch::new(
        collect_hybrid_gi_feedback(renderer, context),
        collect_virtual_geometry_feedback(renderer, context),
    )
}

fn collect_hybrid_gi_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
) -> HybridGiRuntimeFeedback {
    HybridGiRuntimeFeedback::new(
        collect_hybrid_gi_completion(renderer),
        context.hybrid_gi_feedback().cloned(),
    )
}

fn collect_virtual_geometry_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
) -> VirtualGeometryRuntimeFeedback {
    VirtualGeometryRuntimeFeedback::new(
        collect_virtual_geometry_completion(renderer),
        renderer
            .last_virtual_geometry_node_and_cluster_cull_page_request_ids()
            .to_vec(),
        context.virtual_geometry_feedback().cloned(),
    )
}

fn collect_hybrid_gi_completion(renderer: &mut SceneRenderer) -> Option<HybridGiGpuCompletion> {
    renderer
        .take_last_hybrid_gi_gpu_completion_parts()
        .map(|parts| {
            let (
                cache_entries,
                completed_probe_ids,
                completed_trace_region_ids,
                probe_irradiance_rgb,
                probe_trace_lighting_rgb,
                scene_prepare_surface_cache_samples,
            ) = parts.into_parts();
            let scene_prepare_resources =
                scene_prepare_surface_cache_samples.map(|(atlas_samples, capture_samples)| {
                    HybridGiRuntimeScenePrepareResources::new(atlas_samples, capture_samples)
                });
            HybridGiGpuCompletion::new(
                cache_entries,
                completed_probe_ids,
                completed_trace_region_ids,
                probe_irradiance_rgb,
                probe_trace_lighting_rgb,
                scene_prepare_resources,
            )
        })
}

fn collect_virtual_geometry_completion(
    renderer: &mut SceneRenderer,
) -> Option<VirtualGeometryGpuCompletion> {
    renderer
        .take_last_virtual_geometry_gpu_completion_parts()
        .map(|parts| {
            let (page_table_entries, completed_page_assignments, completed_page_replacements) =
                parts.into_parts();
            VirtualGeometryGpuCompletion::new(
                page_table_entries,
                completed_page_assignments,
                completed_page_replacements,
            )
        })
}
