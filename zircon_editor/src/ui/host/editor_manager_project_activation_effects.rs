use std::path::{Path, PathBuf};

use crate::core::hub_link::record_recent_project;
use crate::core::logging::{LogEntry, LogSeverity, LogSource};
use crate::core::project::ProjectPreflightCompositionPlan;
use crate::core::recovery::{ProjectSessionEffect, ProjectSessionEffectLedgerStore};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::now_unix_ms;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::project::{ProjectActivationOperationId, ProjectManifestSummary};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

// Project activation does not have a retained-host frame until M4 first-present commits one.
const UNKNOWN_PROJECT_ACTIVATION_LOG_FRAME: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectActivationRollbackDisposition {
    ReleaseSessionGuard,
    RetainSessionGuardForRecovery,
}

pub(super) struct ProjectActivationFailure {
    error: EditorError,
    rollback: ProjectActivationRollbackDisposition,
}

pub(super) struct ProjectActivationCompletion<T> {
    value: T,
    recent_projection: RecentProjectProjectionRequest,
    operation_id: ProjectActivationOperationId,
}

struct RecentProjectProjectionRequest {
    project_root: PathBuf,
    summary: ProjectManifestSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RecentProjectProjectionDisposition {
    Recorded,
    Deferred { diagnostic: String },
}

impl RecentProjectProjectionDisposition {
    pub(super) fn from_result<T, E>(result: Result<T, E>) -> Self
    where
        E: std::fmt::Display,
    {
        match result {
            Ok(_) => Self::Recorded,
            Err(error) => Self::Deferred {
                diagnostic: error.to_string(),
            },
        }
    }
}

impl<T> ProjectActivationCompletion<T> {
    pub(super) fn new(
        value: T,
        project_root: PathBuf,
        summary: ProjectManifestSummary,
        operation_id: ProjectActivationOperationId,
    ) -> Self {
        Self {
            value,
            recent_projection: RecentProjectProjectionRequest {
                project_root,
                summary,
            },
            operation_id,
        }
    }
}

impl ProjectActivationFailure {
    pub(super) fn releasable(error: EditorError) -> Self {
        Self {
            error,
            rollback: ProjectActivationRollbackDisposition::ReleaseSessionGuard,
        }
    }

    pub(super) fn quarantined(error: EditorError) -> Self {
        Self {
            error,
            rollback: ProjectActivationRollbackDisposition::RetainSessionGuardForRecovery,
        }
    }

    pub(super) const fn retains_session_guard(&self) -> bool {
        matches!(
            self.rollback,
            ProjectActivationRollbackDisposition::RetainSessionGuardForRecovery
        )
    }

    pub(super) fn into_error(self) -> EditorError {
        self.error
    }
}

impl EditorManager {
    pub(super) fn run_project_activation_effect<T>(
        &self,
        ledger: &mut ProjectSessionEffectLedgerStore,
        effect: ProjectSessionEffect,
        activate: impl FnOnce() -> Result<T, EditorError>,
    ) -> Result<T, ProjectActivationFailure> {
        ledger.prepare(effect).map_err(|error| {
            self.rollback_failed_project_activation(
                EditorError::Project(format!(
                    "cannot prepare activation effect `{effect:?}`: {error}"
                )),
                ledger,
            )
        })?;
        let value =
            activate().map_err(|error| self.rollback_failed_project_activation(error, ledger))?;
        ledger.commit(effect).map_err(|error| {
            self.rollback_failed_project_activation(
                EditorError::Project(format!(
                    "cannot commit activation effect `{effect:?}`: {error}"
                )),
                ledger,
            )
        })?;
        Ok(value)
    }

    pub(super) fn complete_project_open(
        &self,
        document: &EditorProjectDocument,
        composition: &ProjectPreflightCompositionPlan,
        ledger: &mut ProjectSessionEffectLedgerStore,
    ) -> Result<(), ProjectActivationFailure> {
        self.run_project_activation_effect(ledger, ProjectSessionEffect::Diagnostics, || {
            self.configure_project_diagnostics(&document.root_path)
        })?;
        self.run_project_activation_effect(ledger, ProjectSessionEffect::ProjectPlugins, || {
            self.apply_project_plugin_manifest(
                &document.root_path,
                &document.manifest,
                composition.approved_project_plugins(),
                composition.allows_native_extensions(),
            )
        })?;
        self.run_project_activation_effect(ledger, ProjectSessionEffect::Documents, || {
            self.initialize_document_journal(&document.root_path)?;
            let activation = self
                .document_lifecycle
                .begin_project_session(&document.root_path);
            self.publish_document_messages(activation.messages);
            Ok(())
        })?;
        Ok(())
    }

    /// Shared history is a post-commit projection, never a prerequisite for the writable session.
    pub(super) fn finalize_project_activation<T>(
        &self,
        completion: ProjectActivationCompletion<T>,
    ) -> T {
        let RecentProjectProjectionRequest {
            project_root,
            summary,
        } = completion.recent_projection;
        let disposition = match ProjectSessionEffectLedgerStore::load(
            &project_root,
            completion.operation_id,
        ) {
            Ok(mut ledger) => {
                let projection = match ledger.prepare(ProjectSessionEffect::RecentProjection) {
                    Ok(()) => match RecentProjectProjectionDisposition::from_result(
                        record_recent_project(&project_root, summary, now_unix_ms()),
                    ) {
                        RecentProjectProjectionDisposition::Recorded => ledger
                            .commit(ProjectSessionEffect::RecentProjection)
                            .map(|()| RecentProjectProjectionDisposition::Recorded)
                            .unwrap_or_else(|error| {
                                RecentProjectProjectionDisposition::Deferred {
                                    diagnostic: format!(
                                        "shared recent-project projection completed, but its session effect ledger could not be committed: {error}"
                                    ),
                                }
                            }),
                        RecentProjectProjectionDisposition::Deferred { diagnostic } => ledger
                            .roll_back(ProjectSessionEffect::RecentProjection)
                            .map(|()| RecentProjectProjectionDisposition::Deferred { diagnostic })
                            .unwrap_or_else(|error| {
                                RecentProjectProjectionDisposition::Deferred {
                                    diagnostic: format!(
                                        "{diagnostic}; additionally failed to roll back its session effect ledger: {error}"
                                    ),
                                }
                            }),
                    },
                    Err(error) => RecentProjectProjectionDisposition::Deferred {
                        diagnostic: format!(
                            "cannot prepare the recent-project session effect ledger after Ready: {error}"
                        ),
                    },
                };
                projection
            }
            Err(error) => RecentProjectProjectionDisposition::Deferred {
                diagnostic: format!(
                    "cannot load the committed session effect ledger after Ready: {error}"
                ),
            },
        };
        if let RecentProjectProjectionDisposition::Deferred { diagnostic } = disposition {
            self.emit_deferred_recent_project_projection(&project_root, diagnostic);
        }
        completion.value
    }

    fn emit_deferred_recent_project_projection(&self, project_root: &Path, diagnostic: String) {
        let entry = LogEntry::new(
            LogSource::editor(),
            LogSeverity::Warning,
            format!(
                "shared recent-project projection deferred for `{}`: {diagnostic}",
                ProjectPaths::display_path(project_root).display(),
            ),
            UNKNOWN_PROJECT_ACTIVATION_LOG_FRAME,
            None,
        )
        .or_else(|_| {
            LogEntry::new(
                LogSource::editor(),
                LogSeverity::Warning,
                "shared recent-project projection deferred; diagnostic exceeds the log-entry limit.",
                UNKNOWN_PROJECT_ACTIVATION_LOG_FRAME,
                None,
            )
        });
        if let Ok(entry) = entry {
            let _ = self.context().logs().emit(entry);
        }
    }

    fn rollback_failed_project_activation(
        &self,
        activation_error: EditorError,
        ledger: &mut ProjectSessionEffectLedgerStore,
    ) -> ProjectActivationFailure {
        self.clear_document_journal();
        self.clear_project_plugin_status();
        let registrations = self.plugin_manager().clear_project_registration_reports();
        let close_result = self
            .host
            .roll_back_project_activation(ledger.project_root());
        if matches!(
            close_result.as_ref(),
            Ok(receipt) if receipt.disposition().is_terminal()
        ) {
            self.context().logs().disable_rolling_file();
            self.context().settings().clear_project_layer();
        }
        match (close_result, registrations) {
            (Ok(runtime_receipt), Ok(receipt))
                if runtime_receipt.disposition().is_terminal() && receipt.is_terminal() =>
            {
                match ledger
                .roll_back_active_effects()
                .and_then(|()| ledger.finish_aborted_activation())
            {
                Ok(()) => ProjectActivationFailure::releasable(activation_error),
                Err(ledger_error) => {
                    ProjectActivationFailure::quarantined(EditorError::Project(format!(
                        "project activation failed: {activation_error}; runtime compensation completed but its session effect ledger could not record terminal rollback state: {ledger_error}; the exclusive project session guard remains held for recovery"
                    )))
                }
                }
            }
            (Err(close_error), _) => {
                let ledger_detail = ledger
                    .mark_recovery_required(ProjectSessionEffect::Runtime)
                    .err()
                    .map(|ledger_error| {
                        format!(
                            "; additionally failed to record session-effect recovery state: {ledger_error}"
                        )
                    })
                    .unwrap_or_default();
                ProjectActivationFailure::quarantined(EditorError::Project(format!(
                    "project activation failed: {activation_error}; additionally failed to roll back the runtime project: {close_error}{ledger_detail}; the exclusive project session guard remains held for recovery"
                )))
            }
            (Ok(runtime_receipt), Ok(receipt)) => {
                let runtime_terminal = runtime_receipt.disposition().is_terminal();
                let effect = if runtime_terminal {
                    ProjectSessionEffect::ProjectPlugins
                } else {
                    ProjectSessionEffect::Runtime
                };
                let ledger_detail = ledger
                    .mark_recovery_required(effect)
                    .err()
                    .map(|ledger_error| {
                        format!(
                            "; additionally failed to record session-effect recovery state: {ledger_error}"
                        )
                    })
                    .unwrap_or_default();
                ProjectActivationFailure::quarantined(EditorError::Project(format!(
                    "project activation failed: {activation_error}; runtime rollback disposition {:?}; project-native registrations remain after manager generation {} / catalog generation {}: {:?}{ledger_detail}; the exclusive project session guard remains held for recovery",
                    runtime_receipt.disposition(),
                    receipt.manager_generation(),
                    receipt.catalog_generation(),
                    receipt.remaining_project_package_ids(),
                )))
            }
            (Ok(runtime_receipt), Err(registration_error)) => {
                let runtime_terminal = runtime_receipt.disposition().is_terminal();
                let effect = if runtime_terminal {
                    ProjectSessionEffect::ProjectPlugins
                } else {
                    ProjectSessionEffect::Runtime
                };
                let ledger_detail = ledger
                    .mark_recovery_required(effect)
                    .err()
                    .map(|ledger_error| {
                        format!(
                            "; additionally failed to record session-effect recovery state: {ledger_error}"
                        )
                    })
                    .unwrap_or_default();
                ProjectActivationFailure::quarantined(EditorError::Project(format!(
                    "project activation failed: {activation_error}; runtime rollback disposition {:?}; additionally failed to clear project-native registrations: {registration_error}{ledger_detail}; the exclusive project session guard remains held for recovery",
                    runtime_receipt.disposition(),
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::editor_error::EditorError;
    use super::{ProjectActivationFailure, RecentProjectProjectionDisposition};

    #[test]
    fn failed_recent_project_projection_is_deferred_without_becoming_an_activation_failure() {
        assert_eq!(
            RecentProjectProjectionDisposition::from_result::<(), _>(Err(
                "shared registry is unavailable"
            )),
            RecentProjectProjectionDisposition::Deferred {
                diagnostic: "shared registry is unavailable".to_string(),
            }
        );
    }

    #[test]
    fn incomplete_project_activation_rollback_retains_the_exclusive_guard() {
        let complete = ProjectActivationFailure::releasable(EditorError::Project(
            "activation rollback completed".to_string(),
        ));
        let incomplete = ProjectActivationFailure::quarantined(EditorError::Project(
            "runtime project rollback failed".to_string(),
        ));

        assert!(!complete.retains_session_guard());
        assert!(incomplete.retains_session_guard());
    }

    #[test]
    fn activation_effects_cross_the_durable_gate_before_ready() {
        let effects = include_str!("editor_manager_project_activation_effects.rs");
        let session = include_str!("editor_manager_project_session.rs");
        let gate_start = effects
            .find("fn run_project_activation_effect<T>(")
            .expect("activation effect gate");
        let gate_end = effects[gate_start..]
            .find("fn complete_project_open(")
            .map(|offset| gate_start + offset)
            .expect("activation effect gate boundary");
        let gate = &effects[gate_start..gate_end];
        let prepare = gate
            .find("ledger.prepare(effect)")
            .expect("effect must be durable-prepared");
        let activate = gate
            .find("activate()")
            .expect("effect must execute after preparation");
        let commit = gate
            .find("ledger.commit(effect)")
            .expect("effect must be durable-committed");
        assert!(prepare < activate && activate < commit);

        for effect in [
            "ProjectSessionEffect::Runtime",
            "ProjectSessionEffect::Diagnostics",
            "ProjectSessionEffect::ProjectPlugins",
            "ProjectSessionEffect::Documents",
            "ProjectSessionEffect::UserInterface",
        ] {
            assert!(
                effects.contains(effect) || session.contains(effect),
                "activation effect `{effect}` must pass through the durable gate"
            );
        }

        let admission_start = session
            .find("fn admit_project_session<T>(")
            .expect("project admission owner");
        let admission = &session[admission_start..];
        let session_prepared = admission
            .find("ledger.prepare(ProjectSessionEffect::Session)")
            .expect("session must be ledger-prepared");
        let ready = admission
            .find("guard.commit_ready()")
            .expect("ready generation commit");
        let session_committed = admission
            .find("ledger.commit(ProjectSessionEffect::Session)")
            .expect("session must be ledger-committed");
        assert!(session_prepared < ready && ready < session_committed);
    }

    #[test]
    fn recent_project_projection_runs_after_ready_commit_and_outside_project_open_commit_gate() {
        let effects = include_str!("editor_manager_project_activation_effects.rs");
        let session = include_str!("editor_manager_project_session.rs");
        let activate_start = session
            .find("fn activate_prepared_project<T>(")
            .expect("project activation owner");
        let activate_end = session[activate_start..]
            .find("fn activate_project_from_preflight<T>(")
            .map(|offset| activate_start + offset)
            .expect("preflight-project activation boundary");
        let activate = &session[activate_start..activate_end];
        let admission_call = activate
            .find(".admit_project_session")
            .expect("project activation must commit through admission");
        let recent_projection = activate
            .find("self.finalize_project_activation(completion)")
            .expect("recent projection must run after the committed activation result");
        let admission_start = session
            .find("fn admit_project_session<T>(")
            .expect("project admission owner");
        let admission = &session[admission_start..];
        assert!(admission_call < recent_projection);
        assert!(
            admission.contains("guard.commit_ready()"),
            "admission must commit Ready before its caller may finalize projections"
        );
        let complete_open_start = effects
            .find("fn complete_project_open(")
            .expect("project-open completion owner");
        let complete_open_end = effects[complete_open_start..]
            .find("fn finalize_project_activation")
            .map(|offset| complete_open_start + offset)
            .expect("post-commit projection boundary");
        assert!(
            !effects[complete_open_start..complete_open_end].contains("record_recent_project("),
            "recent history must not participate in the project-open commit gate"
        );
        assert!(
            effects.contains("fn finalize_project_activation")
                && effects.contains("ProjectSessionEffect::RecentProjection"),
            "recent history is a separately tracked post-Ready projection"
        );
    }

    #[test]
    fn project_activation_consumes_only_preflight_approved_plugin_capabilities() {
        let source = include_str!("editor_manager_project_activation_effects.rs");
        let complete_start = source
            .find("fn complete_project_open(")
            .expect("project-open completion owner");
        let complete_end = source[complete_start..]
            .find("fn finalize_project_activation")
            .map(|offset| complete_start + offset)
            .expect("post-open projection boundary");
        let complete = &source[complete_start..complete_end];

        assert!(complete.contains("composition.approved_project_plugins()"));
        assert!(complete.contains("composition.allows_native_extensions()"));
        assert!(!complete.contains("&document.manifest),"));
    }
}
