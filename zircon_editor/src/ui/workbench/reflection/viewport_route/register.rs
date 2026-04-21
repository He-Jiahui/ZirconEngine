use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime::ui::binding::UiEventKind;

use super::super::name_mapping::binding_view_id;
use super::super::route_registration::register_stub_route;
use super::default_command::default_viewport_command;

pub(crate) fn register_viewport_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<zircon_runtime::ui::event_ui::UiRouteId> {
    let default_command = default_viewport_command(action_id)?;
    let path = zircon_runtime::ui::binding::UiEventPath::new(
        binding_view_id(activity),
        "ViewportSurface",
        event_kind,
    );
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::viewport_command(default_command.clone()),
    );
    Some(register_stub_route(service, registration_binding))
}
