use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::scene::scene_renderer::environment::{
    RealtimeIblPendingSubmission, RealtimeIblRuntime,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, TransientResourcePool,
};
use crate::graphics::types::GraphicsError;

pub(super) fn ensure_compiled_scene_graph_resources(
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    has_full_post_process_resources: bool,
    has_scene_clear_resources: bool,
) -> Result<(), GraphicsError> {
    if !deferred_lighting_profile.supports_compiled_scene_graph() {
        return Err(GraphicsError::Asset(
            "environment-only PBR startup profile cannot execute a compiled scene graph".to_owned(),
        ));
    }
    if !has_full_post_process_resources {
        return Err(GraphicsError::Asset(
            "compiled scene graph requires full post-process resources".to_owned(),
        ));
    }
    if !has_scene_clear_resources {
        return Err(GraphicsError::Asset(
            "compiled scene graph requires scene-clear resources".to_owned(),
        ));
    }
    Ok(())
}

/// Returns transient backings after an error leaves an unsubmitted graph frame.
pub(super) fn abort_compiled_scene_graph_resource_frame(
    graph_resources: &mut RenderGraphExecutionResources,
    transient_resource_pool: &mut TransientResourcePool,
) {
    graph_resources.release_transient_backings_into_pool(transient_resource_pool);
    transient_resource_pool.end_frame();
}

pub(super) fn abort_realtime_ibl_submission(
    realtime_ibl: &mut RealtimeIblRuntime,
    submission: &mut Option<RealtimeIblPendingSubmission>,
) {
    if let Some(submission) = submission.take() {
        realtime_ibl.complete_submission(submission, false);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RenderGenerationIds {
    frame: u64,
    pub(super) mesh_commands: u64,
}

impl RenderGenerationIds {
    pub(super) fn new(frame: u64, mesh_commands: u64) -> Self {
        Self {
            frame,
            mesh_commands,
        }
    }

    pub(super) fn timer_frame(self) -> u64 {
        self.frame
    }
}
