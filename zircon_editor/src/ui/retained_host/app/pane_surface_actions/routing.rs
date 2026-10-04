use super::super::build_export_actions;

pub(super) fn is_build_export_surface_action(control_id: &str, action_id: &str) -> bool {
    control_id == build_export_actions::BUILD_EXPORT_ACTION_CONTROL_ID
        || build_export_actions::parse_build_export_action(action_id).is_some()
}

#[cfg(test)]
#[path = "tests/routing.rs"]
mod tests;
