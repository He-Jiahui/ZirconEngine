use crate::ui::{EditorActivityReflection, EditorUiControlService};

use super::action_route::register_action_route;

pub(super) fn register_activity_routes(
    service: &mut EditorUiControlService,
    activity: &mut EditorActivityReflection,
) {
    let activity_meta = activity.clone();
    for action in &mut activity.actions {
        register_action_route(service, &activity_meta, action);
    }
}
