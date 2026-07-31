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
mod tests {
    use super::update_optional_stat_string;

    #[test]
    fn optional_stat_string_reuses_storage_for_stable_and_shorter_values() {
        let mut value = Some(String::from("default-render-profile"));
        let allocation = value.as_ref().unwrap().as_ptr();

        update_optional_stat_string(&mut value, Some("default-render-profile"));
        assert_eq!(value.as_ref().unwrap().as_ptr(), allocation);

        update_optional_stat_string(&mut value, Some("low"));
        assert_eq!(value.as_deref(), Some("low"));
        assert_eq!(value.as_ref().unwrap().as_ptr(), allocation);
    }
}
