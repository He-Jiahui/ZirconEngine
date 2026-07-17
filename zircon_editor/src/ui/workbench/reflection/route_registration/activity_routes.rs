use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;

use super::action_route::register_action_route;

pub(super) fn register_activity_routes(
    service: &mut EditorUiControlService,
    activity: &mut EditorActivityReflection,
) {
    let mut actions = std::mem::take(&mut activity.actions);
    for action in &mut actions {
        register_action_route(service, activity, action);
    }
    activity.actions = actions;
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn route_registration_does_not_clone_the_activity_projection() {
        let source = include_str!("activity_routes.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(!implementation.contains("activity.clone()"));
        assert!(implementation.contains("std::mem::take(&mut activity.actions)"));
    }
}
