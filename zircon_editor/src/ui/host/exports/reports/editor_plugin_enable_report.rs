use zircon_runtime::core::framework::project::ProjectPluginSelection;

use super::super::super::editor_capabilities::EditorCapabilitySnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginEnableReport {
    pub plugin_id: String,
    pub enabled: bool,
    pub project_selection: ProjectPluginSelection,
    pub editor_capabilities: Vec<String>,
    pub capability_snapshot: EditorCapabilitySnapshot,
    pub diagnostics: Vec<String>,
}

use std::path::PathBuf;
use zircon_runtime::core::resource::io::SourceRevision;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginEnablementPhase {
    Prepare,
    DurableCommit,
    RuntimeApply,
    Reconcile,
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginEnablementDurable {
    NotCommitted,
    Durable,
    CleanupDeferred,
    CommitRecoveryDeferred,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginEnablementApplied {
    NotApplied,
    Applied,
    Restored,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginEnablementRecovery {
    None,
    PendingRuntimeApply,
    RestartRequired,
    RuntimeRepairRequired,
    FileRecoveryRequired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginEnablementCauseKind {
    Prepare,
    RevisionChanged,
    IdentityChanged,
    CatalogChanged,
    DurableCommit,
    CapabilityApply,
    LifecycleApply,
    Rollback,
    Panic,
    Acknowledge,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginEnablementCause {
    pub kind: PluginEnablementCauseKind,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginEnablementOperationReport {
    pub operation_id: String,
    pub project_root: PathBuf,
    pub project_instance: String,
    pub session_generation: u64,
    pub plugin_id: String,
    pub requested_enabled: bool,
    pub expected_revision: Option<SourceRevision>,
    pub committed_revision: Option<SourceRevision>,
    pub phase: PluginEnablementPhase,
    pub durable: PluginEnablementDurable,
    pub applied: PluginEnablementApplied,
    pub recovery: PluginEnablementRecovery,
    pub primary_cause: Option<PluginEnablementCause>,
    pub rollback_cause: Option<PluginEnablementCause>,
    pub detail: Option<EditorPluginEnableReport>,
    pub recovery_admission: Option<PluginEnablementAdmission>,
}
impl PluginEnablementOperationReport {
    pub fn status_message(&self) -> String {
        let state = match (self.durable, self.applied, self.recovery, self.phase) {
            (
                _,
                PluginEnablementApplied::NotApplied,
                PluginEnablementRecovery::None,
                PluginEnablementPhase::Complete,
            ) => "no pending plugin change",
            (PluginEnablementDurable::NotCommitted, _, _, _) => "change was not committed",
            (PluginEnablementDurable::CommitRecoveryDeferred, _, _, _) => {
                "file commit outcome requires recovery"
            }
            (_, _, recovery, _) => match recovery {
                PluginEnablementRecovery::None => "change saved and applied",
                PluginEnablementRecovery::PendingRuntimeApply => {
                    "change saved; runtime apply is pending"
                }
                PluginEnablementRecovery::RuntimeRepairRequired => {
                    "change saved; close the project and resolve before reopening"
                }
                PluginEnablementRecovery::FileRecoveryRequired => {
                    "project file recovery is required"
                }
                PluginEnablementRecovery::RestartRequired => {
                    "pending change reverted; project may be reopened"
                }
            },
        };
        format!(
            "Plugin {}: {}{}",
            self.plugin_id,
            state,
            self.primary_cause
                .as_ref()
                .map(|c| format!(": {}", c.message))
                .unwrap_or_default()
        )
    }
}

#[derive(Clone)]
pub struct PluginEnablementAdmission(
    std::sync::Arc<crate::core::recovery::SessionAdmissionRequest>,
);
impl PluginEnablementAdmission {
    pub(crate) fn new(request: crate::core::recovery::SessionAdmissionRequest) -> Self {
        Self(std::sync::Arc::new(request))
    }
    pub(crate) fn request(&self) -> &crate::core::recovery::SessionAdmissionRequest {
        self.0.as_ref()
    }
}
impl std::fmt::Debug for PluginEnablementAdmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PluginEnablementAdmission(<host-owned>)")
    }
}
impl PartialEq for PluginEnablementAdmission {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for PluginEnablementAdmission {}
#[derive(Clone, Debug)]
pub enum PluginEnablementRecoveryCommand {
    Inspect {
        admission: Option<PluginEnablementAdmission>,
    },
    RecoverFiles {
        admission: PluginEnablementAdmission,
    },
    Retry {
        operation_id: String,
        expected_revision: zircon_runtime::core::resource::io::SourceRevision,
    },
    Revert {
        operation_id: String,
        expected_revision: zircon_runtime::core::resource::io::SourceRevision,
        admission: Option<PluginEnablementAdmission>,
    },
}
