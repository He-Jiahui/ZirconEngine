use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use zircon_runtime_interface::ui::event_ui::UiRouteId;

pub(crate) fn register_binding_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> UiRouteId {
    service
        .route_id_for_binding(&binding.as_ui_binding())
        .unwrap_or_else(|| service.register_binding_route(binding.as_ui_binding()))
}
