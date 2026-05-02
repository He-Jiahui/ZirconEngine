use crate::ui::binding::{
    inspector_field_control_id, DraftCommand, EditorUiBinding, EditorUiBindingPayload,
};
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime_interface::ui::{
    binding::{UiBindingValue, UiEventKind, UiEventPath},
    event_ui::UiRouteId,
};

use super::name_mapping::binding_view_id;
use super::route_registration::register_stub_route;

pub(super) fn register_draft_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<UiRouteId> {
    let view_id = binding_view_id(activity);
    let (control_id, payload) = match action_id {
        "edit_field" => (
            inspector_field_control_id("name").unwrap_or("NameField"),
            EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                subject_path: "entity://selected".to_string(),
                field_id: "name".to_string(),
                value: UiBindingValue::string(String::new()),
            }),
        ),
        "set_mesh_import_path" => (
            "MeshImportPathEdited",
            EditorUiBindingPayload::draft_command(DraftCommand::SetMeshImportPath {
                value: String::new(),
            }),
        ),
        _ => return None,
    };
    let path = UiEventPath::new(view_id, control_id, event_kind);
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        payload,
    );
    Some(register_stub_route(service, registration_binding))
}
