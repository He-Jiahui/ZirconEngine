use super::super::super::render_framework_state::RenderFrameworkState;

pub(super) fn publish_scene_submission_completion_stats(state: &mut RenderFrameworkState) {
    state.stats.last_scene_submission_completion_report =
        state.renderer.last_scene_submission_completion_report();
}
