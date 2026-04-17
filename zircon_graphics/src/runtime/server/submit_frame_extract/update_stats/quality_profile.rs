use super::super::super::render_server_state::RenderServerState;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn update_quality_profile(
    state: &mut RenderServerState,
    context: &FrameSubmissionContext,
) {
    if let Some(profile) = context.quality_profile.clone() {
        state.stats.last_quality_profile = Some(profile);
    }
}
