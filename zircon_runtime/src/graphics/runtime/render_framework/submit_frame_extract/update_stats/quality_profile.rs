use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn update_quality_profile(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
) {
    if let Some(profile) = context.quality_profile() {
        update_optional_stat_string(&mut state.stats.last_quality_profile, Some(profile));
    }
}

pub(super) fn update_optional_stat_string(target: &mut Option<String>, value: Option<&str>) {
    match value {
        Some(value) => match target {
            Some(current) if current != value => {
                current.clear();
                current.push_str(value);
            }
            Some(_) => {}
            None => *target = Some(value.to_owned()),
        },
        None => {
            *target = None;
        }
    }
}

#[cfg(test)]
#[path = "tests/quality_profile.rs"]
mod tests;
