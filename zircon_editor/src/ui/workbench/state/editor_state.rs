use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::context::EditorContext;
use crate::core::editing::authoring_world::{AuthoringWorldAccessError, EditorAuthoringWorld};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::SelectionSnapshot;
use crate::core::editor_message::DocumentId;
use crate::core::logging::{LogEntry, LogSeverity, LogSource};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::SceneViewportController;
use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::snapshot::{
    EditorBridgeDiagnosticsSnapshot, SceneEntryProjectionCache, StatusTaskProgressSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::console_history::EditorConsoleHistory;
use super::editor_state_play_mode::EditorPlaySession;
use super::EditorStateOperationError;
use crate::core::editing::interactive_transform::InteractiveTransformSession;
use zircon_runtime_interface::math::Vec3;

/// UI state which must be restored if a binding batch fails after changing selection.
///
/// The workbench owns the private console and gizmo-adjacent state, so dispatchers receive this
/// checkpoint instead of reimplementing partial restoration at each binding boundary.
pub(crate) struct InspectorBindingUiCheckpoint {
    selection: SelectionModel,
    transaction_selection: SelectionSnapshot,
    name_field: String,
    parent_field: String,
    transform_fields: [String; 3],
    scale_fields: [String; 3],
    dynamic_fields: BTreeMap<String, String>,
    orbit_target: Vec3,
    status_line: String,
    console_history: EditorConsoleHistory,
}
/// Editor shell state shared between the UI host and runtime scene inspection.
pub struct EditorState {
    pub(crate) context: Arc<EditorContext>,
    pub(crate) world: EditorAuthoringWorld,
    pub(crate) viewport_controller: SceneViewportController,
    pub(crate) name_field: String,
    pub(crate) parent_field: String,
    pub(crate) transform_fields: [String; 3],
    pub(crate) scale_fields: [String; 3],
    pub(crate) inspector_dynamic_fields: BTreeMap<String, String>,
    pub(crate) mesh_import_path: String,
    pub(crate) asset_workspace: AssetWorkspaceState,
    pub(crate) project_path: String,
    pub(crate) session_mode: EditorSessionMode,
    pub(crate) welcome: WelcomePaneSnapshot,
    pub(crate) project_open: bool,
    pub(crate) status_line: String,
    pub(in crate::ui::workbench) console_history: EditorConsoleHistory,
    pub(crate) status_task_progress: Option<StatusTaskProgressSnapshot>,
    pub(crate) bridge_diagnostics: EditorBridgeDiagnosticsSnapshot,
    /// A binding copied from the committed document lifecycle transition, not a second identity
    /// authority. Scene commands may only target this document-local history.
    pub(crate) active_scene_document: Option<DocumentId>,
    pub(in crate::ui::workbench) scene_entry_projection_cache: SceneEntryProjectionCache,
    pub(in crate::ui::workbench) interactive_transform: Option<InteractiveTransformSession>,
    pub(crate) play_session: Option<EditorPlaySession>,
    #[cfg(test)]
    pub(crate) fail_next_transaction_selection_sync: bool,
    #[cfg(test)]
    pub(crate) fail_next_inspector_checkpoint_restore: bool,
}

impl EditorState {
    /// Projects a non-interactive authoring-world failure without turning it into a fake empty
    /// scene. The facade suppresses duplicates until a healthy access succeeds.
    pub(crate) fn report_authoring_world_access_failure(
        &self,
        surface: &'static str,
        error: &AuthoringWorldAccessError,
    ) {
        if !self.world.should_report_access_failure(error) {
            return;
        }
        let entry = LogEntry::new(
            LogSource::runtime(),
            LogSeverity::Error,
            format!("editor {surface} lost authoring-world access: {error}"),
            0,
            None,
        );
        if let Ok(entry) = entry {
            let _ = self.context.logs().emit(entry);
        }
    }

    pub(crate) fn transactions(&self) -> &crate::core::editing::engine::EditorTransactionEngine {
        self.context.transactions()
    }

    pub(crate) fn ensure_inspector_binding_can_begin(
        &self,
    ) -> Result<(), EditorStateOperationError> {
        if self.interactive_transform.is_some() || self.viewport_controller.is_handle_drag_active()
        {
            return Err(EditorStateOperationError::InspectorBindingActiveGizmo);
        }
        Ok(())
    }

    pub(crate) fn has_active_gizmo_interaction(&self) -> bool {
        self.interactive_transform.is_some() || self.viewport_controller.is_handle_drag_active()
    }

    pub(crate) fn inspector_binding_ui_checkpoint(
        &self,
    ) -> Result<InspectorBindingUiCheckpoint, EditorStateOperationError> {
        let transaction_selection = self
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)?
            .ok_or(EditorStateOperationError::TransactionContextMissing)?;
        Ok(InspectorBindingUiCheckpoint {
            selection: self.viewport_controller.selection().clone(),
            transaction_selection,
            name_field: self.name_field.clone(),
            parent_field: self.parent_field.clone(),
            transform_fields: self.transform_fields.clone(),
            scale_fields: self.scale_fields.clone(),
            dynamic_fields: self.inspector_dynamic_fields.clone(),
            orbit_target: self.viewport_controller.orbit_target(),
            status_line: self.status_line.clone(),
            console_history: self.console_history.clone(),
        })
    }

    pub(crate) fn restore_inspector_binding_ui_checkpoint(
        &mut self,
        checkpoint: InspectorBindingUiCheckpoint,
    ) -> Result<(), EditorStateOperationError> {
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_inspector_checkpoint_restore) {
            return Err(EditorStateOperationError::InspectorCheckpointRestoreFailed);
        }
        let InspectorBindingUiCheckpoint {
            selection,
            transaction_selection,
            name_field,
            parent_field,
            transform_fields,
            scale_fields,
            dynamic_fields,
            orbit_target,
            status_line,
            console_history,
        } = checkpoint;
        *self.viewport_controller.selection_mut() = selection;
        self.name_field = name_field;
        self.parent_field = parent_field;
        self.transform_fields = transform_fields;
        self.scale_fields = scale_fields;
        self.inspector_dynamic_fields = dynamic_fields;
        self.viewport_controller.set_orbit_target(orbit_target);
        self.status_line = status_line;
        self.console_history = console_history;
        self.transactions()
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.restore_selection_snapshot(&transaction_selection)
            })?
            .ok_or(EditorStateOperationError::TransactionContextMissing)??;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fail_next_inspector_checkpoint_restore_for_test(&mut self) {
        self.fail_next_inspector_checkpoint_restore = true;
    }
}

impl Drop for EditorState {
    fn drop(&mut self) {
        self.viewport_controller.shutdown_scene_modes();
    }
}
