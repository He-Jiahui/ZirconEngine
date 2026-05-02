use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime_interface::ui::event_ui::UiActionDescriptor;

use super::super::animation_route::register_animation_route;
use super::super::asset_route::register_asset_route;
use super::super::docking_route::register_docking_route;
use super::super::draft_route::register_draft_route;
use super::super::inspector_route::register_inspector_route;
use super::super::viewport_route::register_viewport_route;

pub(super) fn register_action_route(
    service: &mut EditorUiControlService,
    activity_meta: &EditorActivityReflection,
    action: &mut UiActionDescriptor,
) {
    if action.route_id.is_some() {
        action.callable_from_remote = true;
        return;
    }

    let route_id = match action.action_id.as_str() {
        "focus_view" | "detach_to_window" => register_docking_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "create_animation_track" => register_animation_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "apply_batch" => register_inspector_route(service, activity_meta, action.event_kind),
        "edit_field" | "set_mesh_import_path" => register_draft_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "import_model" => register_asset_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "pointer_move" | "left_press" | "left_release" | "right_press" | "right_release"
        | "middle_press" | "middle_release" | "scroll" | "resize" => register_viewport_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        _ => None,
    };

    if let Some(route_id) = route_id {
        action.route_id = Some(route_id);
        action.callable_from_remote = true;
    }
}
