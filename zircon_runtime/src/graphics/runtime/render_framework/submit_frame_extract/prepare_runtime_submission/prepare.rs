use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use crate::{HybridGiRuntimePrepareInput, VirtualGeometryRuntimePrepareInput};

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn prepare_runtime_submission(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> PreparedRuntimeSubmission {
    let hybrid_gi_evictable_probe_ids = prepare_hybrid_gi_runtime(state, viewport, context)
        .map(|output| output.into_evictable_probe_ids())
        .unwrap_or_default();
    let virtual_geometry_evictable_page_ids =
        prepare_virtual_geometry_runtime(state, viewport, context)
            .map(|output| output.into_evictable_page_ids())
            .unwrap_or_default();

    PreparedRuntimeSubmission::new(
        hybrid_gi_evictable_probe_ids,
        virtual_geometry_evictable_page_ids,
    )
}

fn prepare_hybrid_gi_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Option<crate::HybridGiRuntimePrepareOutput> {
    if !context.hybrid_gi_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtime();
        }
        return None;
    }

    let Some(registration) = state.hybrid_gi_runtime_provider.clone() else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtime();
        }
        return None;
    };
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked while building frame submission context");
    let input = HybridGiRuntimePrepareInput::new(
        context.hybrid_gi_extract(),
        context.scene_meshes(),
        context.scene_directional_lights(),
        context.scene_point_lights(),
        context.scene_spot_lights(),
        context.hybrid_gi_update_plan(),
        context.predicted_generation(),
    );
    Some(
        record
            .ensure_hybrid_gi_runtime(registration.provider())
            .prepare_frame(input),
    )
}

fn prepare_virtual_geometry_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Option<crate::VirtualGeometryRuntimePrepareOutput> {
    if !context.virtual_geometry_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtime();
        }
        return None;
    }

    let Some(registration) = state.virtual_geometry_runtime_provider.clone() else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtime();
        }
        return None;
    };
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked while building frame submission context");
    let visibility_context = context.visibility_context();
    let input = VirtualGeometryRuntimePrepareInput::new(
        context.virtual_geometry_extract(),
        context.virtual_geometry_page_upload_plan(),
        &visibility_context.virtual_geometry_visible_clusters,
        &visibility_context.virtual_geometry_draw_segments,
        context.predicted_generation(),
    );
    Some(
        record
            .ensure_virtual_geometry_runtime(registration.provider())
            .prepare_frame(input),
    )
}
