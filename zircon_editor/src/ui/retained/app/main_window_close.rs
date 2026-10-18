use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::editing::engine::SceneCloseAdmission;
use crate::core::extension::DocumentCloseAdmission;
use crate::core::jobs::EditorJobShutdownState;
use crate::core::recovery::{FinalAutosaveDrain, ProjectSessionEffect};
use crate::core::settings::SettingsPersistenceShutdown;
use crate::ui::host::{
    CloseAutosavePayloadBudget, ProjectCloseCoordinatorPhase, ProjectCloseOperation,
};
use zircon_runtime::core::runtime::tasks::BoundedKeyedIoShutdownGuard;

use super::close_prompt::{self, DirtyCloseView, PendingClosePrompt};
use super::project_close::RetainedProjectCloseError;
use super::RetainedEditorHost;

const CLOSE_POLL_INTERVAL: Duration = Duration::from_millis(25);
const CLOSE_DRAIN_BUDGET: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainWindowClosePhase {
    Preparing,
    WaitingDecision,
    FinalAutosave,
    Jobs,
    Settings,
    Complete,
    RecoveryRequired,
}

/// One normal main-window close owns every pending operation through its real
/// terminal outcome. In particular, dropping a Pending settings guard may wait.
pub(super) struct OwnedMainWindowClose {
    phase: MainWindowClosePhase,
    activation: Option<ProjectCloseOperation>,
    operation: Option<ProjectCloseOperation>,
    approval: PendingClosePrompt,
    discard: bool,
    project_root: Option<PathBuf>,
    scene: Option<SceneCloseAdmission>,
    documents: Option<DocumentCloseAdmission>,
    captured_dirty: Vec<DirtyCloseView>,
    payloads: Arc<CloseAutosavePayloadBudget>,
    autosave: Option<FinalAutosaveDrain>,
    settings: Option<SettingsPersistenceShutdown>,
    failed_settings_guard: Option<BoundedKeyedIoShutdownGuard>,
    deadline: Instant,
    failure: Option<String>,
    exit_requested: bool,
}

impl OwnedMainWindowClose {
    fn blocks_session_ui(&self) -> bool {
        self.scene.is_some()
            || matches!(
                self.phase,
                MainWindowClosePhase::RecoveryRequired | MainWindowClosePhase::Complete
            )
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn main_window_close_blocks_session_ui(&self) -> bool {
        self.main_window_close
            .as_ref()
            .is_some_and(OwnedMainWindowClose::blocks_session_ui)
    }

    pub(in crate::ui::retained_host::app) fn main_window_close_waiting_decision(&self) -> bool {
        self.main_window_close
            .as_ref()
            .is_some_and(|close| close.phase == MainWindowClosePhase::WaitingDecision)
    }

    pub(in crate::ui::retained_host::app) fn main_window_close_requested(&self) -> bool {
        self.main_window_close.is_some()
    }

    pub(in crate::ui::retained_host::app) fn main_window_close_complete(&self) -> bool {
        self.main_window_close
            .as_ref()
            .is_some_and(|close| close.phase == MainWindowClosePhase::Complete)
    }

    pub(in crate::ui::retained_host::app) fn main_window_close_poll_deadline(
        &self,
    ) -> Option<Instant> {
        self.main_window_close
            .as_ref()
            .filter(|close| close.phase != MainWindowClosePhase::Complete)
            .map(|_| Instant::now() + CLOSE_POLL_INTERVAL)
    }

    fn main_window_close_decision_is_reversible(&self, close: &OwnedMainWindowClose) -> bool {
        let coordinator_is_reversible = match self.project_close_coordinator.phase() {
            ProjectCloseCoordinatorPhase::Decision => {
                self.project_close_coordinator.operation().is_none()
            }
            ProjectCloseCoordinatorPhase::Closed => close.activation.is_none(),
            _ => false,
        };
        coordinator_is_reversible
            && close.operation.is_none()
            && close.scene.is_none()
            && close.documents.is_none()
            && close.autosave.is_none()
            && close.settings.is_none()
            && close.failed_settings_guard.is_none()
            && close.failure.is_none()
    }

    /// Cancel can retire an owned decision only before any close effect starts.
    /// An actual Closing operation may already exist when quiesce returned Pending.
    pub(in crate::ui::retained_host::app) fn cancel_owned_main_window_close_decision(
        &mut self,
    ) -> bool {
        let Some(mut close) = self.main_window_close.take() else {
            return true;
        };
        if close.phase == MainWindowClosePhase::WaitingDecision
            && self.main_window_close_decision_is_reversible(&close)
        {
            self.ui.set_lifecycle_frame_update(Some(Instant::now()));
            return true;
        }
        self.fail_owned_main_window_close(
            &mut close,
            "Close effects have started; Cancel cannot restore this Session. Recovery is required."
                .into(),
        );
        self.main_window_close = Some(close);
        false
    }

    pub(in crate::ui::retained_host::app) fn begin_owned_main_window_close(
        &mut self,
        approval: PendingClosePrompt,
        discard: bool,
    ) {
        let prepare_input = self.main_window_close.as_ref().is_none_or(|close| {
            close.phase == MainWindowClosePhase::WaitingDecision
                && self.main_window_close_decision_is_reversible(close)
        });
        if prepare_input {
            self.native_window_focus_lost();
            self.cancel_viewport_interaction();
        }
        if let Some(mut close) = self.main_window_close.take() {
            // A new decision cannot replace an operation whose real coordinator
            // already entered Closing, even if an earlier poll returned Pending.
            if close.phase == MainWindowClosePhase::WaitingDecision {
                if self.main_window_close_decision_is_reversible(&close) {
                    close.approval = approval;
                    close.discard = discard;
                    close.deadline = Instant::now() + CLOSE_DRAIN_BUDGET;
                    close.phase = MainWindowClosePhase::Preparing;
                } else {
                    self.fail_owned_main_window_close(
                        &mut close,
                        "Close effects have started; the previous decision cannot be replaced."
                            .into(),
                    );
                }
            }
            self.main_window_close = Some(close);
            self.ui.set_lifecycle_frame_update(Some(Instant::now()));
            return;
        }
        let activation = match self.editor_manager.enter() {
            Ok(manager) => manager.project_close_activation_identity(),
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let project_root = activation
            .as_ref()
            .map(|operation| operation.project_root().to_path_buf());
        self.main_window_close = Some(OwnedMainWindowClose {
            phase: MainWindowClosePhase::Preparing,
            activation,
            operation: None,
            approval,
            discard,
            project_root,
            scene: None,
            documents: None,
            captured_dirty: Vec::new(),
            payloads: Arc::new(CloseAutosavePayloadBudget::default()),
            autosave: None,
            settings: None,
            failed_settings_guard: None,
            deadline: Instant::now() + CLOSE_DRAIN_BUDGET,
            failure: None,
            exit_requested: false,
        });
        self.ui.set_lifecycle_frame_update(Some(Instant::now()));
    }

    /// Runs before all normal tick early returns and all session/projection work.
    /// Polling never drops an unfinished guard or invokes synchronous job shutdown.
    pub(in crate::ui::retained_host::app) fn poll_owned_main_window_close(&mut self) {
        let Some(mut close) = self.main_window_close.take() else {
            return;
        };
        if close.phase != MainWindowClosePhase::Complete {
            self.ui
                .set_lifecycle_frame_update(Some(Instant::now() + CLOSE_POLL_INTERVAL));
        }
        if close.phase == MainWindowClosePhase::RecoveryRequired {
            // Late outcomes are retained even after a failed deadline. No new
            // close generation, save batch, or settings fence is issued here.
            if let Some(drain) = close.autosave.as_mut() {
                self.runtime.context().autosave().poll_final_autosave(drain);
            }
            if let Some(owner) = close.settings.as_ref() {
                let _ = owner.poll_result();
            }
            if let Some(owner) = close.failed_settings_guard.as_ref() {
                let _ = owner.poll_result();
            }
        } else if close.phase != MainWindowClosePhase::Complete
            && close.phase != MainWindowClosePhase::WaitingDecision
        {
            if Instant::now() >= close.deadline {
                self.fail_owned_main_window_close(&mut close,
                    "Main-window close deadline expired; pending owners and the Session are retained.".into());
            } else if let Err(message) = self.advance_owned_main_window_close(&mut close) {
                self.fail_owned_main_window_close(&mut close, message);
            }
        }
        let exit = close.phase == MainWindowClosePhase::Complete && !close.exit_requested;
        if exit {
            close.exit_requested = true;
        }
        self.main_window_close = Some(close);
        if exit {
            self.ui.request_exit();
        }
    }

    fn advance_owned_main_window_close(
        &mut self,
        close: &mut OwnedMainWindowClose,
    ) -> Result<(), String> {
        match close.phase {
            MainWindowClosePhase::Preparing => {
                let current = self
                    .editor_manager
                    .enter()
                    .map_err(|error| error.to_string())?
                    .project_close_activation_identity();
                if current != close.activation {
                    return Err(
                        "Project activation changed; the previous close decision cannot be reused."
                            .into(),
                    );
                }
                // Review a stale decision before begin_close retires the heartbeat
                // or commits any focus, asset, UI, or Play effect. A later drift
                // belongs to Recovery, never a reversible Save/Discard/Cancel plan.
                if self.main_window_close_decision_is_reversible(close) {
                    let dirty = self.main_window_dirty_views()?;
                    let scene_token = self.dirty_project_scene_token()?;
                    let decision_valid = if close.discard {
                        close.approval.permits_discard(&dirty, scene_token.as_ref())
                    } else {
                        dirty.is_empty() && scene_token.is_none()
                    };
                    if !decision_valid {
                        close.approval.finish_save(dirty, scene_token);
                        self.begin_close_prompt_plan(close.approval.clone());
                        close.phase = MainWindowClosePhase::WaitingDecision;
                        self.set_status_line(
                            "Changes were updated before closing; review Save, Discard, or Cancel.",
                        );
                        return Ok(());
                    }
                }
                let quiesce_result =
                    self.quiesce_project_close_for_main_window(close.activation.clone());
                // Quiesce may already own Closing and committed effects when it
                // reports Pending. Capture that real ownership on every boundary.
                if !matches!(
                    self.project_close_coordinator.phase(),
                    ProjectCloseCoordinatorPhase::Decision | ProjectCloseCoordinatorPhase::Closed
                ) {
                    close.operation = self.project_close_coordinator.operation().cloned();
                }
                let operation = match quiesce_result {
                    Ok(operation) => operation,
                    Err(error) if close_preparation_is_pending(&error) => {
                        self.set_status_line(error.to_string());
                        return Ok(());
                    }
                    Err(error) => return Err(error.to_string()),
                };
                if operation != close.activation {
                    return Err(
                        "The close coordinator no longer owns the approved activation.".into(),
                    );
                }
                close.operation = operation;

                // Both admissions are try-only. An existing edit/save/capture
                // keeps the host alive; the UI thread never waits for it.
                let scene = match self.runtime.context().transactions().try_close_admission() {
                    Ok(scene) => scene,
                    Err(error) => {
                        self.set_status_line(error.to_string());
                        return Ok(());
                    }
                };
                let documents = match self
                    .editor_manager
                    .enter()
                    .map_err(|error| error.to_string())?
                    .try_document_close_admission()
                {
                    Ok(documents) => documents,
                    Err(error) => {
                        self.set_status_line(error.to_string());
                        return Ok(());
                    }
                };
                // Install both idle capabilities before any fallible recheck.
                // A late revision or observation failure must retain the fence
                // and actual Closing owner rather than reopen ordinary editing.
                close.scene = Some(scene);
                close.documents = Some(documents);
                let dirty = self.main_window_dirty_views()?;
                let scene_token = self.dirty_project_scene_token()?;
                let decision_valid = if close.discard {
                    close.approval.permits_discard(&dirty, scene_token.as_ref())
                } else {
                    dirty.is_empty() && scene_token.is_none()
                };
                if !decision_valid {
                    return Err("Changes were updated after close effects started; the Session and mutation fences are retained for recovery.".into());
                }
                close.captured_dirty = dirty;
                let requests = if close.discard {
                    None
                } else {
                    Some(
                        self.final_autosave_requests_for_main_window(
                            close.operation.as_ref(),
                            close.scene.as_ref().expect("scene admission installed"),
                            close
                                .documents
                                .as_ref()
                                .expect("document admission installed"),
                            &close.captured_dirty,
                            Arc::clone(&close.payloads),
                        )?,
                    )
                };
                close.autosave = Some(self.runtime.context().autosave().begin_final_autosave(
                    close.project_root.as_deref(),
                    requests,
                    close.deadline,
                ));
                close.phase = MainWindowClosePhase::FinalAutosave;
            }
            MainWindowClosePhase::FinalAutosave => {
                let drain = close
                    .autosave
                    .as_mut()
                    .expect("final autosave owner installed");
                self.runtime.context().autosave().poll_final_autosave(drain);
                if let Some(failure) = drain.failure() {
                    return Err(failure.message().to_owned());
                }
                if !drain.ready_to_shutdown_jobs() {
                    return Ok(());
                }
                if close.payloads.retained_bytes() != 0 {
                    return Err("A final payload outlived its completed write.".into());
                }
                self.runtime.context().jobs().request_shutdown();
                close.phase = MainWindowClosePhase::Jobs;
            }
            MainWindowClosePhase::Jobs => {
                let progress = self.runtime.context().jobs().shutdown_progress();
                match progress.state() {
                    EditorJobShutdownState::NotRequested | EditorJobShutdownState::Pending => {
                        return Ok(())
                    }
                    EditorJobShutdownState::Failed => {
                        return Err(progress
                            .cleanup_failure()
                            .unwrap_or("Editor jobs failed during shutdown.")
                            .to_owned())
                    }
                    EditorJobShutdownState::Quiescent => {}
                }
                if self.main_window_dirty_views()? != close.captured_dirty
                    || !close.approval.permits_discard(
                        &close.captured_dirty,
                        self.dirty_project_scene_token()?.as_ref(),
                    )
                {
                    return Err(
                        "A document or scene revision changed after the final close capture."
                            .into(),
                    );
                }
                let manager = self
                    .editor_manager
                    .enter()
                    .map_err(|error| error.to_string())?;
                if manager.project_close_activation_identity() != close.activation {
                    return Err("Project activation changed before close commit.".into());
                }
                if let Some(operation) = close.operation.as_ref() {
                    manager
                        .validate_project_close_operation(operation, ProjectSessionEffect::Session)
                        .map_err(|error| error.to_string())?;
                }
                drop(manager);
                if let Some(operation) = close.operation.clone() {
                    // Runtime/doc/workspace effects commit once. The Session
                    // lease is deliberately retained through Settings success.
                    self.commit_quiesced_project_close(
                        operation,
                        close.scene.as_ref(),
                        close.documents.as_ref(),
                        true,
                    )
                    .map_err(|error| error.to_string())?;
                }
                if !self
                    .runtime
                    .context()
                    .autosave()
                    .release_drained_shutdown_adapters()
                {
                    return Err("Autosave adapters still own shutdown work.".into());
                }
                match self
                    .runtime
                    .context()
                    .settings_mutations()
                    .flush_then_shutdown()
                {
                    Ok(owner) => close.settings = Some(owner),
                    Err(error) => {
                        // Failed submission still owns a real lane drain, but
                        // raw quiescence cannot turn a failed flush into success.
                        close.failed_settings_guard =
                            Some(self.runtime.context().settings_mutations().shutdown());
                        return Err(error.to_string());
                    }
                }
                close.phase = MainWindowClosePhase::Settings;
            }
            MainWindowClosePhase::Settings => {
                match close
                    .settings
                    .as_ref()
                    .expect("settings owner installed")
                    .poll_result()
                {
                    None => return Ok(()),
                    Some(Err(error)) => return Err(error.to_string()),
                    Some(Ok(_)) => {}
                }
                // The borrowed success proves real worker completion AND the
                // successful fence before consuming legacy finish.
                close
                    .settings
                    .take()
                    .expect("successful settings owner")
                    .finish()
                    .map_err(|error| error.to_string())?;
                if let Some(error) = self.ui.take_first_presented_frame_capture_error() {
                    return Err(error);
                }
                if let Some(operation) = close.operation.as_ref() {
                    self.finish_quiesced_project_close(operation)
                        .map_err(|error| error.to_string())?;
                }
                close.phase = MainWindowClosePhase::Complete;
            }
            MainWindowClosePhase::WaitingDecision
            | MainWindowClosePhase::Complete
            | MainWindowClosePhase::RecoveryRequired => {}
        }
        Ok(())
    }

    fn main_window_dirty_views(&self) -> Result<Vec<DirtyCloseView>, String> {
        let documents = self
            .editor_manager
            .enter()
            .map_err(|error| error.to_string())?
            .dirty_document_toolkits()
            .map_err(|error| error.to_string())?;
        Ok(close_prompt::all_dirty_close_views(&documents))
    }

    fn fail_owned_main_window_close(&mut self, close: &mut OwnedMainWindowClose, message: String) {
        if close.failure.is_some() {
            return;
        }
        if let Some(operation) = close.operation.as_ref() {
            let _ = self.project_close_coordinator.require_recovery(operation);
            if let Ok(manager) = self.editor_manager.enter() {
                let _ = manager.require_project_close_recovery(
                    operation,
                    ProjectSessionEffect::Session,
                    message.clone(),
                );
            }
        }
        close.failure = Some(message.clone());
        close.phase = MainWindowClosePhase::RecoveryRequired;
        self.set_status_line(message);
    }
}

fn close_preparation_is_pending(error: &RetainedProjectCloseError) -> bool {
    matches!(
        error,
        RetainedProjectCloseError::PendingDocumentSave { .. }
            | RetainedProjectCloseError::QueuedDocumentSaveAll
            | RetainedProjectCloseError::PendingModelImport
            | RetainedProjectCloseError::PendingAssetRelocation
            | RetainedProjectCloseError::PendingAssetDeletion
            | RetainedProjectCloseError::PlayTeardown { .. }
            | RetainedProjectCloseError::PendingPlayEditDecision
    )
}
