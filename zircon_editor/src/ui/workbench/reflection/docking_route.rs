use crate::ui::EditorActivityReflection;
use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use crate::ui::control::EditorUiControlService;
use zircon_runtime_interface::ui::{
    binding::{UiEventKind, UiEventPath},
    event_ui::UiRouteId,
};

use super::name_mapping::binding_view_id;
use super::route_registration::register_stub_route;

pub(super) fn register_docking_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<UiRouteId> {
    let view_id = binding_view_id(activity);
    let control_id = match action_id {
        "workbench.view.focus" => "FocusViewButton",
        "workbench.view.detach_to_window" => "DetachViewButton",
        _ => return None,
    };
    let path = UiEventPath::new(view_id, control_id, event_kind);
    let default_command = default_dock_command(activity, action_id)?;
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::dock_command(default_command.clone()),
    );
    Some(register_stub_route(service, registration_binding))
}

fn default_dock_command(
    activity: &EditorActivityReflection,
    action_id: &str,
) -> Option<DockCommand> {
    match action_id {
        "workbench.view.focus" => Some(DockCommand::FocusView {
            instance_id: activity.instance_id.clone(),
        }),
        "workbench.view.detach_to_window" => Some(DockCommand::DetachViewToWindow {
            instance_id: activity.instance_id.clone(),
            window_id: format!("window:{}", activity.instance_id),
        }),
        _ => None,
    }
}
