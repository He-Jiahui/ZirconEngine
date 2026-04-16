use zircon_editor_ui::{EditorActivityReflection, EditorUiBinding, EditorUiControlService};

use super::asset_route::register_asset_route;
use super::docking_route::register_docking_route;
use super::draft_route::register_draft_route;
use super::inspector_route::register_inspector_route;
use super::viewport_route::register_viewport_route;

pub fn register_workbench_reflection_routes(
    service: &mut EditorUiControlService,
    mut model: zircon_editor_ui::EditorWorkbenchReflectionModel,
) -> zircon_editor_ui::EditorWorkbenchReflectionModel {
    for item in &mut model.menu_items {
        if item.route_id.is_some() {
            continue;
        }
        item.route_id = Some(register_menu_route(service, item.binding.clone()));
    }

    for page in &mut model.pages {
        for activity in &mut page.activities {
            register_activity_routes(service, activity);
        }
    }
    for drawer in &mut model.drawers {
        for activity in &mut drawer.activities {
            register_activity_routes(service, activity);
        }
    }
    for window in &mut model.floating_windows {
        for activity in &mut window.activities {
            register_activity_routes(service, activity);
        }
    }

    model
}

fn register_menu_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> zircon_ui::UiRouteId {
    register_stub_route(service, binding)
}

fn register_activity_routes(
    service: &mut EditorUiControlService,
    activity: &mut EditorActivityReflection,
) {
    let activity_meta = activity.clone();
    for action in &mut activity.actions {
        if action.route_id.is_some() {
            action.callable_from_remote = true;
            continue;
        }

        let route_id = match action.action_id.as_str() {
            "focus_view" | "detach_to_window" => register_docking_route(
                service,
                &activity_meta,
                action.action_id.as_str(),
                action.event_kind,
            ),
            "apply_batch" => register_inspector_route(service, &activity_meta, action.event_kind),
            "edit_field" | "set_mesh_import_path" => register_draft_route(
                service,
                &activity_meta,
                action.action_id.as_str(),
                action.event_kind,
            ),
            "import_model" => register_asset_route(
                service,
                &activity_meta,
                action.action_id.as_str(),
                action.event_kind,
            ),
            "pointer_move" | "left_press" | "left_release" | "right_press" | "right_release"
            | "middle_press" | "middle_release" | "scroll" | "resize" => register_viewport_route(
                service,
                &activity_meta,
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
}

pub(super) fn register_stub_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> zircon_ui::UiRouteId {
    service
        .route_id_for_binding(&binding.as_ui_binding())
        .unwrap_or_else(|| service.register_route_stub(binding.as_ui_binding()))
}
