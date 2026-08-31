use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::asset::project::ProjectPaths;

use super::{
    AutosaveError, AutosaveRecoveryCatalogDiagnostic, AutosaveStore,
    ProjectSessionAdmissionRecordV1, ProjectSessionEffect, ProjectSessionEffectLedgerError,
    ProjectSessionEffectLedgerPhase, ProjectSessionEffectLedgerStore,
    ProjectSessionEffectRecoveryEntry, ProjectSessionRecoveryStatus, RestoreFlow, RestoreFlowError,
    RestoreStartup, SessionGuard, SessionGuardError, SessionLockInspection,
};

/// The recovery policy applied to the session effect ledger referenced by a residual session lock.
///
/// Only a terminal ledger proves that every session effect is settled. Missing evidence is
/// deliberately treated as uncertain rather than as evidence that nothing ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectRecoveryTakeoverDisposition {
    TerminalLedger,
    OperatorReconciliationRequired(ProjectRecoveryReconciliationReason),
}

impl ProjectRecoveryTakeoverDisposition {
    pub(crate) fn from_status(status: &ProjectSessionRecoveryStatus) -> Self {
        match status {
            ProjectSessionRecoveryStatus::Terminal => Self::TerminalLedger,
            ProjectSessionRecoveryStatus::Missing => Self::OperatorReconciliationRequired(
                ProjectRecoveryReconciliationReason::MissingSessionEffectLedger,
            ),
            ProjectSessionRecoveryStatus::Incomplete { phase, effects } => {
                Self::OperatorReconciliationRequired(
                    ProjectRecoveryReconciliationReason::IncompleteSessionEffects {
                        phase: *phase,
                        effects: effects.clone(),
                    },
                )
            }
            ProjectSessionRecoveryStatus::RecoveryRequired { phase, effects } => {
                Self::OperatorReconciliationRequired(
                    ProjectRecoveryReconciliationReason::RecoveryRequiredSessionEffects {
                        phase: *phase,
                        effects: effects.clone(),
                    },
                )
            }
        }
    }

    pub(crate) const fn allows_takeover(&self) -> bool {
        matches!(self, Self::TerminalLedger)
    }

    pub(crate) fn operator_reconciliation_detail(&self) -> Option<String> {
        let Self::OperatorReconciliationRequired(reason) = self else {
            return None;
        };
        Some(reason.detail())
    }
}

/// The reason an operator must reconcile a residual project before it can be reopened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectRecoveryReconciliationReason {
    MissingSessionEffectLedger,
    IncompleteSessionEffects {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffectRecoveryEntry>,
    },
    RecoveryRequiredSessionEffects {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffectRecoveryEntry>,
    },
}

impl ProjectRecoveryReconciliationReason {
    pub(crate) fn effects(&self) -> Option<&[ProjectSessionEffectRecoveryEntry]> {
        match self {
            Self::MissingSessionEffectLedger => None,
            Self::IncompleteSessionEffects { effects, .. }
            | Self::RecoveryRequiredSessionEffects { effects, .. } => Some(effects),
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::MissingSessionEffectLedger => {
                "its session effect ledger is missing, so prior session effects cannot be proven terminal"
                    .to_string()
            }
            Self::IncompleteSessionEffects { phase, effects } => format!(
                "session phase `{phase:?}` retained effects [{}]",
                session_effect_states(effects),
            ),
            Self::RecoveryRequiredSessionEffects { phase, effects } => format!(
                "session phase `{phase:?}` retained exact effect states [{}]",
                session_effect_states(effects),
            ),
        }
    }
}

/// Read-only recovery data for one project. It never acquires, replaces, or releases a writer
/// lease; the admission owner rechecks this state under its OS lease before it can take over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectRecoveryAssessment {
    project_root: PathBuf,
    admission: ProjectRecoveryAdmission,
    restore_startup: RestoreStartup,
    autosave_diagnostics: Vec<AutosaveRecoveryCatalogDiagnostic>,
}

impl ProjectRecoveryAssessment {
    pub(crate) fn inspect(
        project_root: impl AsRef<Path>,
    ) -> Result<Self, ProjectRecoveryAssessmentError> {
        let requested_path = project_root.as_ref();
        let project_root = ProjectPaths::resolve_path(requested_path)
            .map(|resolved| resolved.into_operation_path())
            .map_err(
                |source| ProjectRecoveryAssessmentError::ResolveProjectRoot {
                    path: requested_path.to_path_buf(),
                    source,
                },
            )?;
        let inspection = SessionGuard::inspect(&project_root).map_err(|source| {
            ProjectRecoveryAssessmentError::InspectSessionLock {
                project_root: project_root.clone(),
                source,
            }
        })?;
        let SessionLockInspection::Residual(residual_lock) = inspection else {
            return Ok(Self {
                project_root,
                admission: ProjectRecoveryAdmission::NoResidualSession,
                restore_startup: RestoreStartup::NoRecoveryNeeded,
                autosave_diagnostics: Vec::new(),
            });
        };
        let session_status = ProjectSessionEffectLedgerStore::inspect_recovery(
            &project_root,
            residual_lock.operation_id(),
        )
        .map_err(
            |source| ProjectRecoveryAssessmentError::InspectSessionEffectLedger {
                project_root: project_root.clone(),
                source,
            },
        )?;
        let disposition = ProjectRecoveryTakeoverDisposition::from_status(&session_status);
        let report = AutosaveStore::new(&project_root)
            .recovery_catalog()
            .map_err(
                |source| ProjectRecoveryAssessmentError::InspectAutosaveCatalog {
                    project_root: project_root.clone(),
                    source,
                },
            )?;
        let restore_startup = RestoreFlow::detect(
            SessionLockInspection::Residual(residual_lock.clone()),
            report.candidates().iter().cloned(),
        )
        .map_err(|source| ProjectRecoveryAssessmentError::BuildRestorePlan {
            project_root: project_root.clone(),
            source,
        })?;
        let admission = match disposition {
            ProjectRecoveryTakeoverDisposition::TerminalLedger => {
                ProjectRecoveryAdmission::TerminalLedgerTakeover { residual_lock }
            }
            ProjectRecoveryTakeoverDisposition::OperatorReconciliationRequired(reason) => {
                ProjectRecoveryAdmission::OperatorReconciliationRequired {
                    residual_lock,
                    reason,
                }
            }
        };
        Ok(Self {
            project_root,
            admission,
            restore_startup,
            autosave_diagnostics: report.diagnostics().to_vec(),
        })
    }

    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) fn admission(&self) -> &ProjectRecoveryAdmission {
        &self.admission
    }

    pub(crate) fn restore_startup(&self) -> &RestoreStartup {
        &self.restore_startup
    }

    pub(crate) fn autosave_diagnostics(&self) -> &[AutosaveRecoveryCatalogDiagnostic] {
        &self.autosave_diagnostics
    }
}

/// The operator-facing session disposition. A terminal record can be taken over explicitly;
/// every other residual state remains locked until its previous effects have been reconciled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectRecoveryAdmission {
    NoResidualSession,
    TerminalLedgerTakeover {
        residual_lock: ProjectSessionAdmissionRecordV1,
    },
    OperatorReconciliationRequired {
        residual_lock: ProjectSessionAdmissionRecordV1,
        reason: ProjectRecoveryReconciliationReason,
    },
}

impl ProjectRecoveryAdmission {
    pub(crate) fn residual_lock(&self) -> Option<&ProjectSessionAdmissionRecordV1> {
        match self {
            Self::NoResidualSession => None,
            Self::TerminalLedgerTakeover { residual_lock }
            | Self::OperatorReconciliationRequired { residual_lock, .. } => Some(residual_lock),
        }
    }

    pub(crate) const fn allows_recovery_takeover(&self) -> bool {
        matches!(self, Self::TerminalLedgerTakeover { .. })
    }

    pub(crate) fn reconciliation_reason(&self) -> Option<&ProjectRecoveryReconciliationReason> {
        match self {
            Self::OperatorReconciliationRequired { reason, .. } => Some(reason),
            Self::NoResidualSession | Self::TerminalLedgerTakeover { .. } => None,
        }
    }

    pub(crate) fn operator_reconciliation_detail(&self) -> Option<String> {
        self.reconciliation_reason()
            .map(ProjectRecoveryReconciliationReason::detail)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ProjectRecoveryAssessmentError {
    #[error("could not resolve project root `{path}`: {source}")]
    ResolveProjectRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not inspect project session lock below `{project_root}`: {source}")]
    InspectSessionLock {
        project_root: PathBuf,
        #[source]
        source: SessionGuardError,
    },
    #[error("could not inspect session effect ledger below `{project_root}`: {source}")]
    InspectSessionEffectLedger {
        project_root: PathBuf,
        #[source]
        source: ProjectSessionEffectLedgerError,
    },
    #[error("could not inspect autosave catalog below `{project_root}`: {source}")]
    InspectAutosaveCatalog {
        project_root: PathBuf,
        #[source]
        source: AutosaveError,
    },
    #[error("could not build restore choices below `{project_root}`: {source}")]
    BuildRestorePlan {
        project_root: PathBuf,
        #[source]
        source: RestoreFlowError,
    },
}

fn session_effect_states(effects: &[ProjectSessionEffectRecoveryEntry]) -> String {
    effects
        .iter()
        .map(|entry| format!("{}={:?}", entry.effect().as_str(), entry.disposition()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{
        ProjectRecoveryTakeoverDisposition, ProjectSessionEffect, ProjectSessionEffectDisposition,
        ProjectSessionEffectLedgerPhase, ProjectSessionEffectRecoveryEntry,
        ProjectSessionRecoveryStatus,
    };

    #[test]
    fn only_a_terminal_ledger_allows_recovery_takeover() {
        assert!(ProjectRecoveryTakeoverDisposition::from_status(
            &ProjectSessionRecoveryStatus::Terminal,
        )
        .allows_takeover());
        assert!(!ProjectRecoveryTakeoverDisposition::from_status(
            &ProjectSessionRecoveryStatus::Missing,
        )
        .allows_takeover());
        assert!(!ProjectRecoveryTakeoverDisposition::from_status(
            &ProjectSessionRecoveryStatus::Incomplete {
                phase: ProjectSessionEffectLedgerPhase::Ready,
                effects: vec![ProjectSessionEffectRecoveryEntry::new(
                    ProjectSessionEffect::Runtime,
                    ProjectSessionEffectDisposition::Committed,
                )],
            },
        )
        .allows_takeover());
        assert!(!ProjectRecoveryTakeoverDisposition::from_status(
            &ProjectSessionRecoveryStatus::RecoveryRequired {
                phase: ProjectSessionEffectLedgerPhase::RecoveryRequired,
                effects: vec![ProjectSessionEffectRecoveryEntry::new(
                    ProjectSessionEffect::Documents,
                    ProjectSessionEffectDisposition::RecoveryRequired,
                )],
            },
        )
        .allows_takeover());
    }
}
