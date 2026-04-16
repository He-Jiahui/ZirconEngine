use zircon_editor_ui::{DraftCommand, EditorUiBinding, EditorUiBindingPayload};

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_editor_binding;

pub(crate) fn dispatch_mesh_import_path_edit(
    runtime: &EditorEventRuntime,
    value: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_editor_binding(
        runtime,
        EditorUiBinding::new(
            "AssetsView",
            "MeshImportPathEdited",
            zircon_editor_ui::EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetMeshImportPath {
                value: value.into(),
            }),
        ),
    )
}
