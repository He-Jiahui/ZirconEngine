use crate::core::play::PlayModeKind;
use crate::core::recovery::ProjectSessionEffect;
use crate::ui::host::{
    DirtyDocumentSaveOwner, EditorError, EditorPlaySessionShutdownReceipt,
    EditorPlayStateShutdownDisposition, ProjectCloseCoordinatorPhase, ProjectCloseError,
    ProjectCloseOperation, ProjectCloseTransitionError, RuntimeEventConsumerShutdownDisposition,
    RuntimePlayGatewayShutdownDisposition, RuntimePlaySessionShutdownDisposition,
};
use crate::ui::retained_host::callback_dispatch::template_bridge::BuiltinHostWindowTemplateBridgeError;
use thiserror::Error;

use super::close_prompt::{self, ClosePromptTarget, PendingClosePrompt};
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ProjectClosePlayShutdownBlocker {
    PluginCleanup { detail: String },
    PlayStop { detail: String },
    EditorStateRestore { detail: String },
    PlayGatewayDetach { detail: String },
    PlayGatewayStillActive { mode: PlayModeKind },
    EventConsumerRetirement { detail: String },
    PendingTerminalTeardown,
}

impl std::fmt::Display for ProjectClosePlayShutdownBlocker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PluginCleanup { detail }
            | Self::PlayStop { detail }
            | Self::EditorStateRestore { detail }
            | Self::PlayGatewayDetach { detail }
            | Self::EventConsumerRetirement { detail } => formatter.write_str(detail),
            Self::PlayGatewayStillActive { mode } => {
                write!(
                    formatter,
                    "Project close retains the active Play gateway in {mode:?} mode"
                )
            }
            Self::PendingTerminalTeardown => formatter
                .write_str("Project close is waiting for Play teardown to reach a terminal state."),
        }
    }
}

#[derive(Debug, Error)]
pub(in crate::ui::retained_host::app) enum RetainedProjectCloseError {
    #[error("Project close is waiting for {owner} to reach a terminal save state.")]
    PendingDocumentSave { owner: DirtyDocumentSaveOwner },
    #[error("Project close is waiting for the queued Save All request to finish.")]
    QueuedDocumentSaveAll,
    #[error("Project close is waiting for the active model import to reach a terminal state.")]
    PendingModelImport,
    #[error("Project close is waiting for the active asset move to reach a terminal state.")]
    PendingAssetRelocation,
    #[error("Project close is waiting for the active asset deletion to reach a terminal state.")]
    PendingAssetDeletion,
    #[error("Project close could not dismiss the command palette: {source}")]
    CommandPalette {
        #[source]
        source: BuiltinHostWindowTemplateBridgeError,
    },
    #[error("{blocker}")]
    PlayTeardown {
        blocker: ProjectClosePlayShutdownBlocker,
    },
    #[error("Project close requires an apply or discard decision for deferred Play edits.")]
    PendingPlayEditDecision,
    #[error("Project close could not release the project session: {source}")]
    Manager {
        #[source]
        source: EditorError,
    },
    #[error("Project close session transaction failed: {source}")]
    ProjectSession {
        #[source]
        source: ProjectCloseError,
    },
    #[error("Project close coordinator rejected its lifecycle transition: {source}")]
    Coordinator {
        #[source]
        source: ProjectCloseTransitionError,
    },
    #[error(
        "Project close released the manager session but could not restore the welcome workspace: {message}"
    )]
    WelcomeWorkspace { message: String },
}

fn project_close_play_shutdown_blocker(
    receipt: &EditorPlaySessionShutdownReceipt,
) -> ProjectClosePlayShutdownBlocker {
    match receipt.play_session() {
        RuntimePlaySessionShutdownDisposition::StoppedWithCleanupFailure { failure, .. } => {
            return ProjectClosePlayShutdownBlocker::PluginCleanup {
                detail: format!(
                    "Project close is waiting for Play plugin cleanup to be repaired: {failure}"
                ),
            };
        }
        RuntimePlaySessionShutdownDisposition::RetirementDeferred { error, .. } => {
            return ProjectClosePlayShutdownBlocker::PlayStop {
                detail: format!("Project close could not stop the active Play session: {error}"),
            };
        }
        RuntimePlaySessionShutdownDisposition::NotPlaying
        | RuntimePlaySessionShutdownDisposition::Stopped { .. } => {}
    }
    match receipt.editor_state() {
        EditorPlayStateShutdownDisposition::RestorationDeferred { error } => {
            return ProjectClosePlayShutdownBlocker::EditorStateRestore {
                detail: format!(
                    "Project close is waiting for editor authoring-state restoration: {error}"
                ),
            };
        }
        EditorPlayStateShutdownDisposition::NotPlaying
        | EditorPlayStateShutdownDisposition::Restored => {}
    }
    match receipt.play_gateway() {
        RuntimePlayGatewayShutdownDisposition::RetirementDeferred { error, .. } => {
            return ProjectClosePlayShutdownBlocker::PlayGatewayDetach {
                detail: format!("Project close could not detach the Play gateway: {error}"),
            };
        }
        RuntimePlayGatewayShutdownDisposition::RetainedForActivePlay { mode } => {
            return ProjectClosePlayShutdownBlocker::PlayGatewayStillActive { mode: *mode };
        }
        RuntimePlayGatewayShutdownDisposition::NotAttached
        | RuntimePlayGatewayShutdownDisposition::Detached { .. } => {}
    }
    match receipt.event_consumers() {
        RuntimeEventConsumerShutdownDisposition::RetirementDeferred { error } => {
            return ProjectClosePlayShutdownBlocker::EventConsumerRetirement {
                detail: format!("Project close could not retire runtime event consumers: {error}"),
            };
        }
        RuntimeEventConsumerShutdownDisposition::NotActive
        | RuntimeEventConsumerShutdownDisposition::Retired
        | RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { .. } => {}
    }
    ProjectClosePlayShutdownBlocker::PendingTerminalTeardown
}

fn welcome_session_after_project_close(
    mut session: EditorStartupSessionDocument,
    closed_root: Option<&std::path::Path>,
) -> EditorStartupSessionDocument {
    session.mode = EditorSessionMode::Welcome;
    session.project = None;
    session.open_builtin_view = None;
    session.status_message = closed_root.map_or_else(
        || "Project was already closed; restored the welcome workspace.".to_string(),
        |root| {
            format!(
                "Closed project {}",
                zircon_runtime::asset::project::ProjectPaths::display_path(root).display()
            )
        },
    );
    session
}

impl RetainedEditorHost {
    /// Starts the project-close plan before any runtime, plugin, layout, or
    /// session teardown. Its participants are the active scene plus every
    /// registered dirty document toolkit.
    pub(in crate::ui::retained_host::app) fn request_project_close(
        &mut self,
    ) -> Result<(), String> {
        if self.project_close_coordinator.phase() == ProjectCloseCoordinatorPhase::Closed
            && self
                .editor_manager
                .active_project_session_focus_target()
                .is_some()
        {
            self.project_close_coordinator
                .reset_for_new_session()
                .map_err(|error| error.to_string())?;
        }
        if let Some(owner) = self.editor_manager.dirty_document_save_owner() {
            self.set_status_line(format!(
                "Project close is waiting for {owner} to finish saving."
            ));
            return Ok(());
        }
        if self.queued_document_save_all {
            self.set_status_line("Project close is waiting for queued Save All.".to_string());
            return Ok(());
        }
        if self.pending_close_prompt.is_some() {
            self.set_status_line("A close decision is already active.".to_string());
            return Ok(());
        }
        self.recompute_if_dirty();
        let dirty_documents = self
            .editor_manager
            .dirty_document_toolkits()
            .map_err(|error| error.to_string())?;
        let dirty_project_scene_generation = self.dirty_project_scene_generation()?;
        if dirty_documents.is_empty() && dirty_project_scene_generation.is_none() {
            return self
                .commit_project_close()
                .map_err(|error| error.to_string());
        }

        let mut prompt = PendingClosePrompt::new(
            ClosePromptTarget::Project,
            Vec::new(),
            close_prompt::all_dirty_close_views(&dirty_documents),
        );
        if let Some(generation) = dirty_project_scene_generation {
            prompt = prompt.with_dirty_project_scene(generation);
        }
        self.begin_close_prompt_plan(prompt);
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn dirty_project_scene_generation(
        &self,
    ) -> Result<Option<u64>, String> {
        let transactions = self.runtime.context().transactions();
        let Some(history_context) = self.runtime.active_scene_history_context() else {
            return Ok(None);
        };
        let is_dirty = transactions
            .is_dirty(history_context)
            .map_err(|error| error.to_string())?;
        if !is_dirty {
            return Ok(None);
        }
        transactions
            .history_generation_snapshot(history_context)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub(in crate::ui::retained_host::app) fn commit_project_close(
        &mut self,
    ) -> Result<(), RetainedProjectCloseError> {
        if let Some(owner) = self.editor_manager.dirty_document_save_owner() {
            return Err(RetainedProjectCloseError::PendingDocumentSave { owner });
        }
        if self.queued_document_save_all {
            return Err(RetainedProjectCloseError::QueuedDocumentSaveAll);
        }

        let operation = match self.project_close_coordinator.phase() {
            ProjectCloseCoordinatorPhase::Decision => {
                let Some(operation) = self
                    .editor_manager
                    .begin_project_close()
                    .map_err(|source| RetainedProjectCloseError::Manager { source })?
                else {
                    let welcome =
                        welcome_session_after_project_close(self.startup_session.clone(), None);
                    return self.apply_startup_session(welcome).map_err(|message| {
                        RetainedProjectCloseError::WelcomeWorkspace { message }
                    });
                };
                self.project_close_coordinator
                    .begin_quiescing(operation.clone())
                    .map_err(|source| RetainedProjectCloseError::Coordinator { source })?;

                if let Err(source) = self
                    .editor_manager
                    .prepare_project_close_effect(&operation, ProjectSessionEffect::FocusBinding)
                {
                    return Err(self.project_close_failure(&operation, source));
                }
                if let Err(message) = self.sync_hub_focus_binding() {
                    let source = self.editor_manager.require_project_close_recovery(
                        &operation,
                        ProjectSessionEffect::FocusBinding,
                        format!("cannot retire the Hub focus binding: {message}"),
                    );
                    return Err(self.project_close_failure(&operation, source));
                }
                if let Err(source) = self
                    .editor_manager
                    .commit_project_close_effect(&operation, ProjectSessionEffect::FocusBinding)
                {
                    return Err(self.project_close_failure(&operation, source));
                }
                operation
            }
            ProjectCloseCoordinatorPhase::Quiescing => self
                .project_close_coordinator
                .operation()
                .cloned()
                .expect("Quiescing close coordinator always owns an operation"),
            ProjectCloseCoordinatorPhase::Committing
            | ProjectCloseCoordinatorPhase::RecoveryRequired => {
                let operation = self
                    .project_close_coordinator
                    .operation()
                    .cloned()
                    .expect("terminal close coordinator always owns an operation");
                let source = self.editor_manager.require_project_close_recovery(
                    &operation,
                    ProjectSessionEffect::Session,
                    "project close cannot resume after entering a terminal in-process phase",
                );
                return Err(self.project_close_failure(&operation, source));
            }
            ProjectCloseCoordinatorPhase::Closed => return Ok(()),
        };

        if let Err(source) = self
            .editor_manager
            .prepare_project_close_effect(&operation, ProjectSessionEffect::AssetJobs)
        {
            return Err(self.project_close_failure(&operation, source));
        }
        if !self.cancel_pending_asset_deletion() {
            return Err(RetainedProjectCloseError::PendingAssetDeletion);
        }
        if !self.cancel_pending_asset_relocation() {
            return Err(RetainedProjectCloseError::PendingAssetRelocation);
        }
        if !self.cancel_pending_model_import() {
            return Err(RetainedProjectCloseError::PendingModelImport);
        }
        self.cancel_pending_active_scene_reload();
        if let Err(source) = self
            .editor_manager
            .commit_project_close_effect(&operation, ProjectSessionEffect::AssetJobs)
        {
            return Err(self.project_close_failure(&operation, source));
        }

        if let Err(source) = self
            .editor_manager
            .prepare_project_close_effect(&operation, ProjectSessionEffect::UserInterface)
        {
            return Err(self.project_close_failure(&operation, source));
        }
        let palette_closed = self
            .workbench_window_bridge
            .close_command_palette()
            .map_err(|source| RetainedProjectCloseError::CommandPalette { source })?;
        self.scene_picker_session = None;
        if palette_closed {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        }
        if let Err(source) = self
            .editor_manager
            .commit_project_close_effect(&operation, ProjectSessionEffect::UserInterface)
        {
            return Err(self.project_close_failure(&operation, source));
        }

        if let Err(source) = self
            .editor_manager
            .prepare_project_close_effect(&operation, ProjectSessionEffect::Play)
        {
            return Err(self.project_close_failure(&operation, source));
        }
        let play_shutdown = self.runtime.shutdown_play_session_for_project_close();
        if !play_shutdown.is_ready_for_project_close() {
            self.sync_activity_notifications();
            return Err(RetainedProjectCloseError::PlayTeardown {
                blocker: project_close_play_shutdown_blocker(&play_shutdown),
            });
        }
        if self
            .runtime
            .play_sessions()
            .pending_edit_decision_prompt()
            .is_some()
        {
            self.sync_activity_notifications();
            return Err(RetainedProjectCloseError::PendingPlayEditDecision);
        }
        if let Err(source) = self
            .editor_manager
            .commit_project_close_effect(&operation, ProjectSessionEffect::Play)
        {
            return Err(self.project_close_failure(&operation, source));
        }

        self.project_close_coordinator
            .begin_committing(&operation)
            .map_err(|source| RetainedProjectCloseError::Coordinator { source })?;
        let committed = match self.editor_manager.commit_project_close(&operation) {
            Ok(committed) => committed,
            Err(source) => return Err(self.project_close_failure(&operation, source)),
        };
        let closed_root = committed.closed_root().map(std::path::Path::to_path_buf);

        let welcome_session = welcome_session_after_project_close(
            self.startup_session.clone(),
            closed_root.as_deref(),
        );
        if let Err(source) = self
            .editor_manager
            .prepare_project_close_effect(&operation, ProjectSessionEffect::WorkspaceProjection)
        {
            return Err(self.project_close_failure(&operation, source));
        }
        if let Err(message) = self.apply_startup_session(welcome_session) {
            let source = self.editor_manager.require_project_close_recovery(
                &operation,
                ProjectSessionEffect::WorkspaceProjection,
                format!("cannot restore the welcome workspace: {message}"),
            );
            return Err(self.project_close_failure(&operation, source));
        }
        if let Err(source) = self
            .editor_manager
            .commit_project_close_effect(&operation, ProjectSessionEffect::WorkspaceProjection)
        {
            return Err(self.project_close_failure(&operation, source));
        }
        if let Err(source) = self.editor_manager.finalize_project_close(&operation) {
            return Err(self.project_close_failure(&operation, source));
        }
        self.project_close_coordinator
            .finish_closed(&operation)
            .map_err(|source| RetainedProjectCloseError::Coordinator { source })?;
        Ok(())
    }

    fn project_close_failure(
        &mut self,
        operation: &ProjectCloseOperation,
        source: ProjectCloseError,
    ) -> RetainedProjectCloseError {
        let _ = self.project_close_coordinator.require_recovery(operation);
        RetainedProjectCloseError::ProjectSession { source }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

    use super::{welcome_session_after_project_close, RetainedProjectCloseError};

    #[test]
    fn successful_close_returns_to_welcome_without_retaining_project_navigation() {
        let mut session = EditorStartupSessionDocument::default();
        session.mode = EditorSessionMode::Project;
        session.open_builtin_view = Some("editor.scene".to_string());

        let welcome =
            welcome_session_after_project_close(session, Some(Path::new("C:/projects/forest")));

        assert_eq!(welcome.mode, EditorSessionMode::Welcome);
        assert!(welcome.project.is_none());
        assert!(welcome.open_builtin_view.is_none());
        assert_eq!(welcome.status_message, "Closed project C:/projects/forest");
    }

    #[test]
    fn close_enters_durable_closing_and_retires_focus_before_quiescing_consumers() {
        let source = include_str!("project_close.rs");
        let close_start = source
            .find("fn commit_project_close(")
            .expect("retained close entry point");
        let close_end = source[close_start..]
            .find("#[cfg(test)]")
            .map(|offset| close_start + offset)
            .expect("retained close test boundary");
        let close = &source[close_start..close_end];
        let begin = close
            .find(".begin_project_close()")
            .expect("durable close admission");
        let focus_sync = close
            .find("self.sync_hub_focus_binding()")
            .expect("focus binding retirement");
        let asset_quiesce = close
            .find("self.cancel_pending_asset_deletion()")
            .expect("asset quiescence");
        let manager_close = close
            .find("self.editor_manager.commit_project_close(&operation)")
            .expect("manager close result");

        assert!(begin < focus_sync);
        assert!(focus_sync < asset_quiesce);
        assert!(asset_quiesce < manager_close);
        assert!(close.contains("ProjectSessionEffect::FocusBinding"));
    }

    #[cfg(windows)]
    #[test]
    fn successful_close_projects_operation_roots_to_a_display_path() {
        let welcome = welcome_session_after_project_close(
            EditorStartupSessionDocument::default(),
            Some(Path::new(r"\\?\C:\projects\forest")),
        );

        assert_eq!(welcome.status_message, r"Closed project C:\projects\forest");
    }

    #[test]
    fn retry_after_committed_runtime_close_still_repairs_the_welcome_surface() {
        let mut session = EditorStartupSessionDocument::default();
        session.mode = EditorSessionMode::Project;
        session.open_builtin_view = Some("editor.asset_browser".to_string());

        let welcome = welcome_session_after_project_close(session, None);

        assert_eq!(welcome.mode, EditorSessionMode::Welcome);
        assert!(welcome.project.is_none());
        assert!(welcome.open_builtin_view.is_none());
        assert_eq!(
            welcome.status_message,
            "Project was already closed; restored the welcome workspace."
        );
    }

    #[test]
    fn project_close_runs_play_teardown_before_releasing_the_project_session() {
        let source = include_str!("project_close.rs");
        let close = source
            .split("fn commit_project_close(&mut self)")
            .nth(1)
            .expect("project-close commit implementation");
        let play_shutdown = close
            .find("shutdown_play_session_for_project_close()")
            .expect("project close must retire Play before project release");
        let project_commit = close
            .find(".commit_project_close(&operation)")
            .expect("project close must commit the manager release");

        assert!(play_shutdown < project_commit);
        assert!(close.contains("is_ready_for_project_close()"));
        assert!(close.contains("pending_edit_decision_prompt()"));
    }

    #[test]
    fn terminal_entry_points_delegate_project_close_to_retained_host() {
        let direct_manager_close = ["editor_manager", "commit_project_close(&"].join(".");

        for source in [include_str!("../app.rs"), include_str!("automation.rs")] {
            assert!(source.contains("host\n        .borrow_mut()\n        .commit_project_close()"));
            assert!(!source.contains(&direct_manager_close));
        }
    }

    #[test]
    fn project_close_errors_keep_terminal_failure_classes_typed() {
        assert!(matches!(
            RetainedProjectCloseError::PendingPlayEditDecision,
            RetainedProjectCloseError::PendingPlayEditDecision
        ));
        assert!(matches!(
            RetainedProjectCloseError::WelcomeWorkspace {
                message: "welcome failed".to_string(),
            },
            RetainedProjectCloseError::WelcomeWorkspace { .. }
        ));
    }
}
