use crate::ui::{EditorUiBinding, EditorUiControlService};

pub(crate) fn register_stub_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> zircon_ui::event_ui::UiRouteId {
    service
        .route_id_for_binding(&binding.as_ui_binding())
        .unwrap_or_else(|| service.register_route_stub(binding.as_ui_binding()))
}
