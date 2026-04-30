use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn update_quality_profile(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
) {
    if let Some(profile) = context.quality_profile() {
        state.stats.last_quality_profile = Some(profile.to_owned());
    }
}
