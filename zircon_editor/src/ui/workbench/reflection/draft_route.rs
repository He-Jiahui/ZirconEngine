use crate::ui::EditorActivityReflection;
use crate::ui::binding::{
    DraftCommand, EditorUiBinding, EditorUiBindingPayload, inspector_field_control_id,
};
use crate::ui::control::EditorUiControlService;
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
        "inspector.field.edit" => (
            inspector_field_control_id("name").unwrap_or("NameField"),
            EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                subject_path: "entity://selected".to_string(),
                field_id: "name".to_string(),
                value: UiBindingValue::string(String::new()),
            }),
        ),
        "workbench.asset.mesh_import.path.set" => (
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
