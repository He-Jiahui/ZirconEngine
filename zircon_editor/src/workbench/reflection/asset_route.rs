use zircon_editor_ui::{
    AssetCommand, EditorActivityReflection, EditorUiBinding, EditorUiBindingPayload,
    EditorUiControlService,
};
use zircon_ui::UiEventKind;

use super::name_mapping::binding_view_id;
use super::route_registration::register_stub_route;

pub(super) fn register_asset_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<zircon_ui::UiRouteId> {
    let (control_id, payload) = match action_id {
        "import_model" => (
            "ImportModel",
            EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
        ),
        _ => return None,
    };
    let path = zircon_ui::UiEventPath::new(binding_view_id(activity), control_id, event_kind);
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        payload,
    );
    Some(register_stub_route(service, registration_binding))
}
