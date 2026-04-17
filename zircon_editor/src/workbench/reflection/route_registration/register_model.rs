use zircon_editor_ui::{EditorUiBinding, EditorUiControlService};

use super::activity_routes::register_activity_routes;
use super::stub_route::register_stub_route;

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
