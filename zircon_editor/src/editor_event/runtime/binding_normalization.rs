use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use crate::editor_event::{
    EditorAssetEvent, EditorDraftEvent, EditorEvent, EditorInspectorEvent, EditorViewportEvent,
};
use crate::{
    dispatch_asset_binding, dispatch_docking_binding, dispatch_draft_binding,
    dispatch_inspector_binding, dispatch_selection_binding, dispatch_viewport_binding,
    dispatch_workbench_binding,
};

pub(super) fn normalize_binding(binding: &EditorUiBinding) -> Result<EditorEvent, String> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { .. } => {
            let crate::WorkbenchHostEvent::Menu(action) =
                dispatch_workbench_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::WorkbenchMenu(action))
        }
        EditorUiBindingPayload::DockCommand(_) => Ok(EditorEvent::Layout(
            dispatch_docking_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::SelectionCommand(_) => Ok(EditorEvent::Selection(
            dispatch_selection_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::AssetCommand(_) => {
            let event = dispatch_asset_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Asset(match event {
                crate::AssetHostEvent::OpenAsset { asset_path } => {
                    EditorAssetEvent::OpenAsset { asset_path }
                }
                crate::AssetHostEvent::SelectFolder { folder_id } => {
                    EditorAssetEvent::SelectFolder { folder_id }
                }
                crate::AssetHostEvent::SelectItem { asset_uuid } => {
                    EditorAssetEvent::SelectItem { asset_uuid }
                }
                crate::AssetHostEvent::ActivateReference { asset_uuid } => {
                    EditorAssetEvent::ActivateReference { asset_uuid }
                }
                crate::AssetHostEvent::SetSearchQuery { query } => {
                    EditorAssetEvent::SetSearchQuery { query }
                }
                crate::AssetHostEvent::SetKindFilter { kind } => {
                    EditorAssetEvent::SetKindFilter { kind }
                }
                crate::AssetHostEvent::SetViewMode { surface, view_mode } => {
                    EditorAssetEvent::SetViewMode { surface, view_mode }
                }
                crate::AssetHostEvent::SetUtilityTab { surface, tab } => {
                    EditorAssetEvent::SetUtilityTab { surface, tab }
                }
                crate::AssetHostEvent::OpenAssetBrowser => EditorAssetEvent::OpenAssetBrowser,
                crate::AssetHostEvent::LocateSelectedAsset => EditorAssetEvent::LocateSelectedAsset,
                crate::AssetHostEvent::ImportModel => EditorAssetEvent::ImportModel,
            }))
        }
        EditorUiBindingPayload::DraftCommand(_) => {
            let event = dispatch_draft_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Draft(match event {
                crate::DraftHostEvent::SetInspectorField {
                    subject_path,
                    field_id,
                    value,
                } => EditorDraftEvent::SetInspectorField {
                    subject_path,
                    field_id,
                    value,
                },
                crate::DraftHostEvent::SetMeshImportPath { value } => {
                    EditorDraftEvent::SetMeshImportPath { value }
                }
            }))
        }
        EditorUiBindingPayload::InspectorFieldBatch { .. } => {
            let batch = dispatch_inspector_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Inspector(EditorInspectorEvent {
                subject_path: batch.subject_path,
                changes: batch.changes,
            }))
        }
        EditorUiBindingPayload::ViewportCommand(_) => {
            Ok(EditorEvent::Viewport(viewport_event_from_binding(binding)?))
        }
        EditorUiBindingPayload::WelcomeCommand(_)
        | EditorUiBindingPayload::PositionOfTrackAndFrame { .. }
        | EditorUiBindingPayload::Custom(_) => Err(format!(
            "unsupported editor event binding {}",
            binding.native_binding()
        )),
    }
}

fn viewport_event_from_binding(binding: &EditorUiBinding) -> Result<EditorViewportEvent, String> {
    let command = dispatch_viewport_binding(binding).map_err(|error| error.to_string())?;
    Ok(crate::host::slint_host::callback_dispatch::viewport_event_from_command(command))
}
