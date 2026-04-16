use zircon_editor_ui::{
    DockCommand, EditorActivityReflection, EditorUiBinding, EditorUiBindingPayload,
    EditorUiControlService,
};
use zircon_ui::UiEventKind;

use super::name_mapping::binding_view_id;
use super::route_registration::register_stub_route;

pub(super) fn register_docking_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<zircon_ui::UiRouteId> {
    let view_id = binding_view_id(activity);
    let control_id = match action_id {
        "focus_view" => "FocusViewButton",
        "detach_to_window" => "DetachViewButton",
        _ => return None,
    };
    let path = zircon_ui::UiEventPath::new(view_id, control_id, event_kind);
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
        "focus_view" => Some(DockCommand::FocusView {
            instance_id: activity.instance_id.clone(),
        }),
        "detach_to_window" => Some(DockCommand::DetachViewToWindow {
            instance_id: activity.instance_id.clone(),
            window_id: format!("window:{}", activity.instance_id),
        }),
        _ => None,
    }
}
