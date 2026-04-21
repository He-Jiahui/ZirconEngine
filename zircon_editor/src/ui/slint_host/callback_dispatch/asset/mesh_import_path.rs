use crate::ui::binding::{DraftCommand, EditorUiBinding, EditorUiBindingPayload};

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

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
            crate::ui::binding::EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetMeshImportPath {
                value: value.into(),
            }),
        ),
    )
}
