use std::path::Path;
use std::time::Instant;

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};

use crate::core::hub_link::publish_focus_signal;
use crate::core::project::{
    ProjectAuthority, ProjectPreflightCompositionPlan, ProjectPreflightReceipt,
    ProjectPreflightRevalidation,
};
use crate::core::recovery::{
    ProjectRecoveryAssessment, ProjectSessionAdmissionRecordV1, ProjectSessionEffect,
    ProjectSessionEffectLedgerStore, ProjectSessionEffectRecoveryEntry,
    ProjectSessionRecoveryStatus, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::EditorStartupSessionDocument;
use zircon_runtime_interface::hub_protocol::HubSessionToken;
use zircon_runtime_interface::project::{
    session_lock::ProjectSessionAdmissionLifecycleV1, ProjectActivationOperationId,
};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use super::editor_manager_project_activation_effects::{
    ProjectActivationCompletion, ProjectActivationFailure,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectSessionAdmissionMode {
    NewSessionOnly,
    RecoveryTakeover,
}

impl EditorManager {
    pub(super) fn open_project_document_with_admission(
        &self,
        preflight: ProjectPreflightReceipt,
        admission: &SessionAdmissionRequest,
    ) -> Result<EditorProjectDocument, EditorError> {
        let _transition = self.begin_project_session_transition()?;
        let (document, recovery) = self.activate_project_from_preflight(
            preflight,
            admission,
            ProjectSessionAdmissionMode::NewSessionOnly,
            Ok,
        )?;
        debug_assert!(recovery.is_none());
        Ok(document)
    }

    pub(super) fn open_project_and_remember_with_session(
        &self,
        preflight: ProjectPreflightReceipt,
        admission: &SessionAdmissionRequest,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let _transition = self.begin_project_session_transition()?;
        let (session, recovery) = self.activate_project_from_preflight(
            preflight,
            admission,
            ProjectSessionAdmissionMode::NewSessionOnly,
            |document| self.host.remember_prepared_project(document),
        )?;
        debug_assert!(recovery.is_none());
        Ok(session)
    }

    /// Opens only through an explicit Recovery profile after a residual lease has been checked.
    pub(super) fn recover_project_and_remember_with_session(
        &self,
        preflight: ProjectPreflightReceipt,
        admission: &SessionAdmissionRequest,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let _transition = self.begin_project_session_transition()?;
        let (session, assessment) = self.activate_project_from_preflight(
            preflight,
            admission,
            ProjectSessionAdmissionMode::RecoveryTakeover,
            |document| self.host.remember_prepared_project(document),
        )?;
        let Some(assessment) = assessment else {
            return Err(self.retain_project_session_for_recovery(EditorError::Project(
                "recovery takeover reached Ready without its lease-validated recovery assessment"
                    .to_string(),
            )));
        };
        self.begin_project_recovery_decisions(
            assessment.project_root(),
            assessment.restore_startup().clone(),
        )
        .map_err(|error| self.retain_project_session_for_recovery(error))?;
        Ok(session)
    }

    pub(super) fn create_project_and_open_with_session(
        &self,
        draft: crate::core::project::NewProjectDraft,
        admission: &SessionAdmissionRequest,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let _transition = self.begin_project_session_transition()?;
        let authority = ProjectAuthority::default();
        let project = authority.create_project(&draft)?;
        let preflight = authority.preflight_project(&project.root)?;
        self.activate_prepared_project(project.into_project(), preflight, admission, |document| {
            self.host.remember_prepared_project(document)
        })
    }

    /// Retains a partially torn-down session under an explicit recovery lifecycle.
    pub(super) fn retain_project_session_for_recovery(&self, source: EditorError) -> EditorError {
        let mut heartbeat = self
            .project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        heartbeat.clear();
        let recovery_detail = match guard_slot.as_mut() {
            Some(guard) => {
                let project_root = guard.project_root().to_path_buf();
                match guard.mark_recovery_required() {
                    Ok(_) => format!(
                        "the exclusive session guard for `{}` remains held for recovery",
                        ProjectPaths::display_path(&project_root).display(),
                    ),
                    Err(recovery_error) => format!(
                        "the exclusive session guard for `{}` remains held, but persisting RecoveryRequired also failed: {recovery_error}",
                        ProjectPaths::display_path(&project_root).display(),
                    ),
                }
            }
            None => {
                "no active project session guard was available to retain for recovery".to_string()
            }
        };
        EditorError::Project(format!("{source}; {recovery_detail}"))
    }

    /// Refreshes the active project-session record when the manager-owned timer is due.
    ///
    /// The retained host may call this at frame cadence, but persistence is bounded by the
    /// manager-local interval. A failure permanently degrades this session rather than allowing
    /// subsequent ticks to keep extending an ownership record whose durability is no longer known.
    pub(crate) fn refresh_project_session_heartbeat_if_due(
        &self,
        now: Instant,
    ) -> Result<(), EditorError> {
        let mut heartbeat = self
            .project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !heartbeat.is_due(now) {
            return Ok(());
        }

        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(guard) = guard_slot.as_mut() else {
            heartbeat.clear();
            return Ok(());
        };
        let project_root = guard.project_root().to_path_buf();
        match guard.refresh_heartbeat() {
            Ok(_) => {
                heartbeat.mark_refreshed(now);
                Ok(())
            }
            Err(error) => {
                heartbeat.mark_degraded();
                Err(EditorError::Project(format!(
                    "project session heartbeat failed for `{}`; the active session is degraded and will not publish another heartbeat: {error}",
                    ProjectPaths::display_path(&project_root).display(),
                )))
            }
        }
    }

    /// Returns the next lifecycle wake for the active, healthy session.
    ///
    /// The retained host maps this to its own wake slot so an idle native window still refreshes
    /// the persisted session record. Degraded and inactive sessions deliberately schedule nothing.
    pub(crate) fn project_session_heartbeat_deadline(&self) -> Option<Instant> {
        self.project_session_heartbeat
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .next_refresh()
    }

    /// Returns the immutable identity consumed by the retained-host Hub focus watcher.
    ///
    /// The watcher receives a snapshot only; it has no capability to mutate the session lock.
    pub(crate) fn active_project_session_focus_target(&self) -> Option<(PathBuf, String, u64)> {
        self.project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .filter(|guard| {
                guard.record().lifecycle() == ProjectSessionAdmissionLifecycleV1::Ready
                    && guard.record().session_generation().is_some()
            })
            .map(|guard| {
                let generation = guard
                    .record()
                    .session_generation()
                    .expect("Ready session guards always carry a committed generation");
                (
                    guard.project_root().to_path_buf(),
                    guard.record().instance_id().to_string(),
                    generation.get(),
                )
            })
    }

    fn take_hub_launch_session(&self) -> Option<HubSessionToken> {
        self.hub_launch_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    fn activate_prepared_project<T>(
        &self,
        project: ProjectManager,
        preflight: ProjectPreflightReceipt,
        admission: &SessionAdmissionRequest,
        finish: impl FnOnce(EditorProjectDocument) -> Result<T, EditorError>,
    ) -> Result<T, EditorError> {
        let project_root = project.paths().root().to_path_buf();
        let completion = self
            .admit_project_session(
                &project_root,
                admission,
                ProjectSessionAdmissionMode::NewSessionOnly,
                |ledger| {
                    self.activate_prepared_project_after_admission(
                        project,
                        preflight.composition(),
                        admission.operation_id(),
                        ledger,
                        finish,
                    )
                },
            )
            .map_err(ProjectActivationFailure::into_error)?;
        let (completion, recovery) = completion;
        debug_assert!(recovery.is_none());
        Ok(self.finalize_project_activation(completion))
    }

    /// The only existing-project materialization path. Canonical path resolution and preflight
    /// are data-only; `ProjectManager` cannot be constructed until the writer lease is held.
    fn activate_project_from_preflight<T>(
        &self,
        preflight: ProjectPreflightReceipt,
        admission: &SessionAdmissionRequest,
        admission_mode: ProjectSessionAdmissionMode,
        finish: impl FnOnce(EditorProjectDocument) -> Result<T, EditorError>,
    ) -> Result<(T, Option<ProjectRecoveryAssessment>), EditorError> {
        let authority = ProjectAuthority::default();
        let completion = self
            .admit_project_session(preflight.root(), admission, admission_mode, |ledger| {
                let revalidated = authority.revalidate_preflight(&preflight).map_err(|error| {
                    ProjectActivationFailure::releasable(error.into())
                })?;
                let ProjectPreflightRevalidation::Unchanged { current } = revalidated else {
                    return Err(ProjectActivationFailure::releasable(EditorError::Project(
                        "project manifest changed after preflight and before admission; retry the launch request"
                            .to_string(),
                    )));
                };
                let project = authority
                    .open_resolved_project(current.resolved_project_path())
                    .map_err(|error| ProjectActivationFailure::releasable(error.into()))?;
                self.activate_prepared_project_after_admission(
                    project.into_project(),
                    current.composition(),
                    admission.operation_id(),
                    ledger,
                    finish,
                )
            })
            .map_err(ProjectActivationFailure::into_error)?;
        let (completion, recovery) = completion;
        Ok((self.finalize_project_activation(completion), recovery))
    }

    fn activate_prepared_project_after_admission<T>(
        &self,
        project: ProjectManager,
        composition: &ProjectPreflightCompositionPlan,
        operation_id: ProjectActivationOperationId,
        ledger: &mut ProjectSessionEffectLedgerStore,
        finish: impl FnOnce(EditorProjectDocument) -> Result<T, EditorError>,
    ) -> Result<ProjectActivationCompletion<T>, ProjectActivationFailure> {
        let document =
            self.run_project_activation_effect(ledger, ProjectSessionEffect::Runtime, || {
                self.host
                    .open_prepared_project(project, composition.allows_scene_restore())
            })?;
        self.complete_project_open(&document, composition, ledger)?;
        let project_root = document.root_path.clone();
        let summary = document.manifest.summary().clone();
        let value = self.run_project_activation_effect(
            ledger,
            ProjectSessionEffect::UserInterface,
            || finish(document),
        )?;
        Ok(ProjectActivationCompletion::new(
            value,
            project_root,
            summary,
            operation_id,
        ))
    }

    fn admit_project_session<T>(
        &self,
        project_root: &Path,
        admission: &SessionAdmissionRequest,
        admission_mode: ProjectSessionAdmissionMode,
        activate: impl FnOnce(
            &mut ProjectSessionEffectLedgerStore,
        ) -> Result<T, ProjectActivationFailure>,
    ) -> Result<(T, Option<ProjectRecoveryAssessment>), ProjectActivationFailure> {
        if self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
        {
            return Err(ProjectActivationFailure::releasable(EditorError::Project(
                format!(
                    "cannot activate `{}` while another editor project session is active",
                    ProjectPaths::display_path(project_root).display()
                ),
            )));
        }

        let hub_launch_session = self.take_hub_launch_session();
        let mut recovery_assessment = None;
        let mut guard = match SessionGuard::claim(project_root, admission).map_err(|error| {
            ProjectActivationFailure::releasable(EditorError::Project(format!(
                "cannot claim the project session for `{}`: {error}",
                ProjectPaths::display_path(project_root).display()
            )))
        })? {
            SessionGuardAdmission::Acquired(mut guard) => {
                if admission_mode == ProjectSessionAdmissionMode::RecoveryTakeover {
                    let recovery_required = ProjectActivationFailure::releasable(
                        EditorError::Project(format!(
                            "recovery profile requires a residual project session for `{}`; use Safe mode to open a project without recovery takeover",
                            ProjectPaths::display_path(project_root).display(),
                        )),
                    );
                    return match guard.release() {
                        Ok(_) => Err(recovery_required),
                        Err(release_error) => Err(ProjectActivationFailure::quarantined(
                            EditorError::Project(format!(
                                "{}; additionally failed to release the newly acquired session guard: {release_error}",
                                recovery_required.into_error(),
                            )),
                        )),
                    };
                }
                guard
            }
            SessionGuardAdmission::Active { record } => {
                if let (Some(session), Some(record)) = (hub_launch_session, record.as_ref()) {
                    if record.lifecycle() == ProjectSessionAdmissionLifecycleV1::Ready
                        && record.session_generation().is_some()
                    {
                        publish_focus_signal(project_root, record, session).map_err(|error| {
                            ProjectActivationFailure::releasable(EditorError::Project(format!(
                                "active project session for `{}` could not receive the Hub focus signal: {error}",
                                ProjectPaths::display_path(project_root).display(),
                            )))
                        })?;
                        return Err(ProjectActivationFailure::releasable(
                            EditorError::HubFocusForwarded {
                                process_id: record.process_id(),
                            },
                        ));
                    }
                }
                return Err(ProjectActivationFailure::releasable(EditorError::Project(
                    active_project_session_message(project_root, record.as_ref()),
                )));
            }
            SessionGuardAdmission::Residual(residual) => match admission_mode {
                ProjectSessionAdmissionMode::NewSessionOnly => {
                    return Err(ProjectActivationFailure::releasable(EditorError::Project(
                        residual_project_session_message(project_root, residual.record()),
                    )));
                }
                ProjectSessionAdmissionMode::RecoveryTakeover => {
                    let assessment = ProjectRecoveryAssessment::inspect(project_root).map_err(
                        |error| {
                        ProjectActivationFailure::releasable(EditorError::Project(format!(
                            "cannot refresh the lease-protected recovery assessment for `{}`: {error}",
                            ProjectPaths::display_path(project_root).display(),
                        )))
                    },
                    )?;
                    let Some(assessed_residual) = assessment.admission().residual_lock() else {
                        return Err(ProjectActivationFailure::releasable(EditorError::Project(
                            "lease-protected recovery assessment no longer contains a residual session"
                                .to_string(),
                        )));
                    };
                    if assessed_residual != residual.record() {
                        return Err(ProjectActivationFailure::releasable(EditorError::Project(
                            format!(
                                "residual project session for `{}` changed before lease-protected recovery assessment completed; retry recovery",
                                ProjectPaths::display_path(project_root).display(),
                            ),
                        )));
                    }
                    if !assessment.admission().allows_recovery_takeover() {
                        let detail = assessment
                            .admission()
                            .operator_reconciliation_detail()
                            .unwrap_or_else(|| {
                                "the residual project session is not terminal".to_string()
                            });
                        return Err(ProjectActivationFailure::releasable(EditorError::Project(
                            recovery_takeover_blocked_message(
                                project_root,
                                residual.record(),
                                &detail,
                            ),
                        )));
                    }
                    let guard = residual.take_over(admission).map_err(|error| {
                        ProjectActivationFailure::releasable(EditorError::Project(format!(
                            "cannot take over the residual project session for `{}`: {error}",
                            ProjectPaths::display_path(project_root).display(),
                        )))
                    })?;
                    recovery_assessment = Some(assessment);
                    guard
                }
            },
        };

        let activation = (|| {
            guard.mark_preflight_approved().map_err(|error| {
                ProjectActivationFailure::releasable(EditorError::Project(format!(
                    "cannot record approved project preflight for `{}`: {error}",
                    ProjectPaths::display_path(project_root).display(),
                )))
            })?;
            guard.begin_activation().map_err(|error| {
                ProjectActivationFailure::releasable(EditorError::Project(format!(
                    "cannot record project activation for `{}`: {error}",
                    ProjectPaths::display_path(project_root).display(),
                )))
            })?;

            ProjectSessionEffectLedgerStore::create(project_root, admission.operation_id())
                .map_err(|error| {
                    ProjectActivationFailure::releasable(EditorError::Project(format!(
                        "cannot create the session effect ledger for `{}`: {error}",
                        ProjectPaths::display_path(project_root).display(),
                    )))
                })
                .and_then(|mut ledger| {
                    let value = activate(&mut ledger)?;
                    ledger
                        .prepare(ProjectSessionEffect::Session)
                        .map_err(|error| {
                            self.rollback_failed_project_activation(
                                EditorError::Project(format!(
                                    "cannot prepare the session effect ledger for `{}`: {error}",
                                    ProjectPaths::display_path(project_root).display(),
                                )),
                                &mut ledger,
                            )
                        })?;
                    Ok((value, ledger))
                })
        })();

        match activation {
            Ok((value, mut ledger)) => {
                if let Err(error) = guard.commit_ready() {
                    let ledger_recovery_error = ledger.require_recovery_for_active_effects().err();
                    let recovery_error = guard.mark_recovery_required().err();
                    let mut guard_slot = self
                        .project_session_guard
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if guard_slot.is_none() {
                        *guard_slot = Some(guard);
                    }
                    let recovery_detail = recovery_error
                        .map(|recovery_error| {
                            format!(
                                "; additionally failed to record recovery state: {recovery_error}"
                            )
                        })
                        .unwrap_or_default();
                    let ledger_recovery_detail = ledger_recovery_error
                        .map(|ledger_recovery_error| {
                            format!(
                                "; additionally failed to record session-effect recovery state: {ledger_recovery_error}"
                            )
                        })
                        .unwrap_or_default();
                    return Err(ProjectActivationFailure::quarantined(EditorError::Project(
                        format!(
                            "project activation for `{}` completed but its Ready generation could not be committed: {error}{recovery_detail}{ledger_recovery_detail}; the exclusive project session guard remains held for recovery",
                            ProjectPaths::display_path(project_root).display(),
                        ),
                    )));
                }
                if let Err(error) = ledger
                    .commit(ProjectSessionEffect::Session)
                    .and_then(|()| ledger.begin_ready())
                {
                    let ledger_recovery_error = ledger.require_recovery_for_active_effects().err();
                    let recovery_error = guard.mark_recovery_required().err();
                    let mut guard_slot = self
                        .project_session_guard
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if guard_slot.is_none() {
                        *guard_slot = Some(guard);
                    }
                    let recovery_detail = recovery_error
                        .map(|recovery_error| {
                            format!(
                                "; additionally failed to record recovery state: {recovery_error}"
                            )
                        })
                        .unwrap_or_default();
                    let ledger_recovery_detail = ledger_recovery_error
                        .map(|ledger_recovery_error| {
                            format!(
                                "; additionally failed to record session-effect recovery state: {ledger_recovery_error}"
                            )
                        })
                        .unwrap_or_default();
                    return Err(ProjectActivationFailure::quarantined(EditorError::Project(
                        format!(
                            "project activation for `{}` reached Ready but its session effect ledger could not be committed: {error}{recovery_detail}{ledger_recovery_detail}; the exclusive project session guard remains held for recovery",
                            ProjectPaths::display_path(project_root).display(),
                        ),
                    )));
                }
                let mut heartbeat = self
                    .project_session_heartbeat
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut guard_slot = self
                    .project_session_guard
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if guard_slot.is_some() {
                    drop(guard_slot);
                    let concurrent_session =
                        ProjectActivationFailure::releasable(EditorError::Project(format!(
                            "project session for `{}` changed while activation was in progress",
                            ProjectPaths::display_path(project_root).display()
                        )));
                    return match guard.release() {
                        Ok(_) => Err(concurrent_session),
                        Err(release_error) => Err(ProjectActivationFailure::quarantined(
                            EditorError::Project(format!(
                                "{}; additionally failed to release the uncommitted session guard: {release_error}",
                                concurrent_session.into_error()
                            )),
                        )),
                    };
                }
                *guard_slot = Some(guard);
                heartbeat.activate(Instant::now());
                Ok((value, recovery_assessment))
            }
            Err(activation_failure) if activation_failure.retains_session_guard() => {
                let recovery_error = guard.mark_recovery_required().err();
                let mut guard_slot = self
                    .project_session_guard
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if guard_slot.is_some() {
                    drop(guard_slot);
                    let activation_error = activation_failure.into_error();
                    return match guard.release() {
                        Ok(_) => Err(ProjectActivationFailure::releasable(EditorError::Project(
                            format!(
                                "project activation failed while another session guard was installed: {activation_error}"
                            ),
                        ))),
                        Err(release_error) => Err(ProjectActivationFailure::quarantined(
                            EditorError::Project(format!(
                                "project activation failed while another session guard was installed: {activation_error}; additionally failed to release the uncommitted session guard: {release_error}"
                            )),
                        )),
                    };
                }
                *guard_slot = Some(guard);
                match recovery_error {
                    None => Err(activation_failure),
                    Some(recovery_error) => Err(ProjectActivationFailure::quarantined(
                        EditorError::Project(format!(
                            "{}; additionally failed to record the RecoveryRequired session state: {recovery_error}",
                            activation_failure.into_error(),
                        )),
                    )),
                }
            }
            Err(activation_failure) => {
                let activation_error = activation_failure.into_error();
                let closed_project_root = guard.project_root().to_path_buf();
                let closed_operation_id = guard.record().operation_id();
                match guard.release() {
                    Ok(_) => {
                        let cleanup = ProjectSessionEffectLedgerStore::load(
                            &closed_project_root,
                            closed_operation_id,
                        )
                        .and_then(|ledger| ledger.cleanup_if_closed());
                        match cleanup {
                            Ok(_) => Err(ProjectActivationFailure::releasable(activation_error)),
                            Err(ledger_error) => Err(ProjectActivationFailure::releasable(
                                EditorError::Project(format!(
                                    "{activation_error}; the session guard was released, but closed session-effect ledger cleanup was deferred: {ledger_error}"
                                )),
                            )),
                        }
                    }
                    Err(release_error) => {
                        let mut guard_slot = self
                            .project_session_guard
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        if guard_slot.is_none() {
                            *guard_slot = Some(guard);
                            return Err(ProjectActivationFailure::quarantined(
                                EditorError::Project(format!(
                                    "project activation failed: {activation_error}; additionally failed to release its session guard: {release_error}; the exclusive project session guard remains held for recovery"
                                )),
                            ));
                        }
                        Err(ProjectActivationFailure::quarantined(EditorError::Project(
                            format!(
                                "project activation failed: {activation_error}; additionally failed to release its uncommitted session guard: {release_error}; another exclusive session guard is already retained for recovery"
                            ),
                        )))
                    }
                }
            }
        }
    }
}

fn active_project_session_message(
    project_root: &Path,
    record: Option<&ProjectSessionAdmissionRecordV1>,
) -> String {
    let project_root = ProjectPaths::display_path(project_root);
    match record {
        Some(record) => format!(
            "project `{}` is already active in editor process {} (instance `{}`)",
            project_root.display(),
            record.process_id(),
            record.instance_id(),
        ),
        None => format!(
            "project `{}` is already active in another editor instance",
            project_root.display(),
        ),
    }
}

fn residual_project_session_message(
    project_root: &Path,
    residual: &ProjectSessionAdmissionRecordV1,
) -> String {
    let display_path = ProjectPaths::display_path(project_root);
    match ProjectSessionEffectLedgerStore::inspect_recovery(project_root, residual.operation_id()) {
        Ok(ProjectSessionRecoveryStatus::Missing) => format!(
            "project session recovery is required for `{}` after editor instance `{}`; no session-effect record was published and the residual lock was preserved",
            display_path.display(),
            residual.instance_id(),
        ),
        Ok(ProjectSessionRecoveryStatus::Terminal) => format!(
            "project session recovery is required for `{}` after editor instance `{}`; its session effects are terminal but the residual lock was preserved for explicit takeover",
            display_path.display(),
            residual.instance_id(),
        ),
        Ok(ProjectSessionRecoveryStatus::Incomplete { phase, effects }) => format!(
            "project session recovery is required for `{}` after editor instance `{}`; session phase `{phase:?}` retained effects [{}] and the residual lock was preserved",
            display_path.display(),
            residual.instance_id(),
            session_effect_states(&effects),
        ),
        Ok(ProjectSessionRecoveryStatus::RecoveryRequired { phase, effects }) => format!(
            "project session recovery is required for `{}` after editor instance `{}`; session phase `{phase:?}` effects [{}] require explicit recovery and the residual lock was preserved",
            display_path.display(),
            residual.instance_id(),
            session_effect_states(&effects),
        ),
        Err(error) => format!(
            "project session recovery is required for `{}` after editor instance `{}`; its session-effect record could not be verified: {error}; the residual lock was preserved",
            display_path.display(),
            residual.instance_id(),
        ),
    }
}

fn recovery_takeover_blocked_message(
    project_root: &Path,
    residual: &ProjectSessionAdmissionRecordV1,
    detail: &str,
) -> String {
    let display_path = ProjectPaths::display_path(project_root);
    format!(
        "recovery profile cannot take over `{}` from editor instance `{}` because {}; the residual exclusive lease remains preserved",
        display_path.display(),
        residual.instance_id(),
        detail,
    )
}

fn session_effect_states(effects: &[ProjectSessionEffectRecoveryEntry]) -> String {
    effects
        .iter()
        .map(|entry| format!("{}={:?}", entry.effect().as_str(), entry.disposition()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "editor_manager_project_session/tests.rs"]
mod tests;
