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
        "workbench.view.focus" | "workbench.view.detach_to_window" => register_docking_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "animation.track.create" => register_animation_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "inspector.apply_batch.invoke" => {
            register_inspector_route(service, activity_meta, action.event_kind)
        }
        "inspector.field.edit" | "workbench.asset.mesh_import.path.set" => register_draft_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "workbench.asset.model.import" => register_asset_route(
            service,
            activity_meta,
            action.action_id.as_str(),
            action.event_kind,
        ),
        "workbench.viewport.pointer.move"
        | "workbench.viewport.pointer.left.press"
        | "workbench.viewport.pointer.left.release"
        | "workbench.viewport.pointer.right.press"
        | "workbench.viewport.pointer.right.release"
        | "workbench.viewport.pointer.middle.press"
        | "workbench.viewport.pointer.middle.release"
        | "workbench.viewport.scroll"
        | "workbench.viewport.resize" => register_viewport_route(
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
