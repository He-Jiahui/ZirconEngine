//! Project transition owns prepare/durable/apply; capability locks never span file I/O.
use super::super::super::editor_manager_project_session::PluginEnablementAdmissionFailure;
use super::super::super::{
    editor_capabilities::EditorCapabilitySnapshot, editor_manager::EditorManager,
};
use super::super::reports::*;
use std::path::{Path, PathBuf};
use zircon_runtime::asset::project::plugin_apply::*;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::core::framework::project::ProjectPluginSelection;
use zircon_runtime::core::resource::io::SourceRevision;
use zircon_runtime::plugin::native::discovery::native_plugin_discovery_generation;
use zircon_runtime::plugin::native::NativePluginLoadReport;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};

pub(super) struct PreparedPluginSelection {
    pub candidate: ProjectManifest,
    pub selection: ProjectPluginSelection,
    pub capabilities: Vec<String>,
    pub previous_enabled: bool,
    pub owns_editor_state: bool,
    pub live_supported: bool,
    pub native_report: NativePluginLoadReport,
}
pub(super) struct RuntimeEnablementBinding {
    pub manager_generation: u64,
    pub catalog_generation: u64,
    pub capabilities: Vec<String>,
    pub snapshot: EditorCapabilitySnapshot,
    pub native_root: PathBuf,
    pub native_generation: Option<u64>,
}
pub(super) struct RuntimeApplyFailure {
    pub kind: PluginEnablementCauseKind,
    pub primary: String,
    pub rollback: Option<String>,
    pub unknown: bool,
}
fn operation(root: &Path, plugin_id: &str, enabled: bool) -> PluginEnablementOperationReport {
    PluginEnablementOperationReport {
        operation_id: String::new(),
        project_root: root.to_path_buf(),
        project_instance: String::new(),
        session_generation: 0,
        plugin_id: plugin_id.to_string(),
        requested_enabled: enabled,
        expected_revision: None,
        committed_revision: None,
        phase: PluginEnablementPhase::Prepare,
        durable: PluginEnablementDurable::NotCommitted,
        applied: PluginEnablementApplied::NotApplied,
        recovery: PluginEnablementRecovery::None,
        primary_cause: None,
        rollback_cause: None,
        detail: None,
        recovery_admission: None,
    }
}
fn failed(
    mut report: PluginEnablementOperationReport,
    kind: PluginEnablementCauseKind,
    cause: impl ToString,
    recovery: PluginEnablementRecovery,
) -> PluginEnablementOperationReport {
    report.primary_cause = Some(PluginEnablementCause {
        kind,
        message: cause.to_string(),
    });
    report.recovery = recovery;
    report
}
impl EditorManager {
    pub fn set_project_plugin_enabled_durable(
        &self,
        root: impl AsRef<Path>,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        self.commit_plugin_enablement(root.as_ref(), plugin_id, enabled, None, None)
    }
    fn capture_enablement_binding(&self, root: &Path) -> Result<RuntimeEnablementBinding, String> {
        let native_root = self.plugin_directory(root);
        let native_generation = native_plugin_discovery_generation(&native_root);
        let _capability = self.lock_editor_capability_updates();
        let state = self.plugin_manager().state_snapshot();
        let configuration = self
            .host
            .runtime_services
            .capability_configuration()
            .map_err(|e| e.to_string())?;
        Ok(RuntimeEnablementBinding {
            manager_generation: state.generation(),
            catalog_generation: state.catalog_generation(),
            capabilities: configuration.enabled_subsystems(),
            snapshot: self.capability_snapshot(),
            native_generation,
            native_root,
        })
    }
    fn commit_plugin_enablement(
        &self,
        root: &Path,
        plugin_id: &str,
        enabled: bool,
        expected: Option<&SourceRevision>,
        supersedes: Option<&str>,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        self.commit_plugin_enablement_with_hooks(
            root,
            plugin_id,
            enabled,
            expected,
            supersedes,
            || (),
            |_| (),
            || (),
        )
    }
    fn commit_plugin_enablement_with_hooks<G>(
        &self,
        root: &Path,
        plugin_id: &str,
        enabled: bool,
        expected: Option<&SourceRevision>,
        supersedes: Option<&str>,
        prepare_checkpoint: impl FnOnce(),
        commit_guard: impl FnOnce(&ProjectPaths) -> G,
        apply_checkpoint: impl FnOnce(),
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let mut report = operation(root, plugin_id, enabled);
        let _transition = self.begin_project_session_transition().map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::IdentityChanged,
                e,
                PluginEnablementRecovery::None,
            )
        })?;
        let paths = ProjectPaths::from_root(root).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::Prepare,
                e,
                PluginEnablementRecovery::None,
            )
        })?;
        let session = self
            .active_project_session_focus_target()
            .filter(|s| s.0 == paths.root())
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::IdentityChanged,
                    "a Ready session for this physical project is required",
                    PluginEnablementRecovery::None,
                )
            })?;
        report.project_root = paths.root().to_path_buf();
        report.project_instance = session.1.clone();
        report.session_generation = session.2;
        // Uncertain effects quarantine the whole Ready session, including a new op/plugin.
        if let Some(fence) =
            self.plugin_enablement_repair_for_session(paths.root(), &session.1, session.2)
        {
            return Err(fence);
        }
        let document = PluginApplyDocument::read(&paths).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::Prepare,
                e,
                PluginEnablementRecovery::FileRecoveryRequired,
            )
        })?;
        report.expected_revision = Some(document.revision.clone());
        if expected.is_some_and(|r| r != &document.revision) {
            return Err(failed(
                report,
                PluginEnablementCauseKind::RevisionChanged,
                "expected committed revision changed",
                PluginEnablementRecovery::PendingRuntimeApply,
            ));
        }
        let (prepared, binding) = self
            .prepare_bound_plugin_selection_with_checkpoint(
                paths.root(),
                &document.manifest,
                plugin_id,
                enabled,
                prepare_checkpoint,
            )
            .map_err(|e| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    e,
                    PluginEnablementRecovery::None,
                )
            })?;
        let old_status = self.published_plugin_status_report();
        let desired_status = self.native_plugin_status_report_from_load_report(
            &prepared.candidate,
            &prepared.native_report,
        );
        let id = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    "operation identities exhausted",
                    PluginEnablementRecovery::None,
                )
            })?;
        report.operation_id = id.nonce().to_string();
        report.phase = PluginEnablementPhase::DurableCommit;
        let receipt = PendingPluginApply {
            version: 1,
            operation_id: report.operation_id.clone(),
            plugin_id: plugin_id.to_string(),
            enabled,
            previous_enabled: prepared.previous_enabled,
            project_guid: document.manifest.project_guid.to_string(),
            manifest_digest: String::new(),
            session_instance: session.1.clone(),
            session_generation: session.2,
            catalog_generation: binding.catalog_generation,
            acknowledged: false,
            resolved_before_activation: false,
        };
        let _commit_guard = commit_guard(&paths);
        // No capability guard is held while either durable document is written.
        let committed =
            commit_plugin_apply(&paths, &document, &prepared.candidate, receipt, supersedes)
                .map_err(|e| {
                    let mut failure = report.clone();
                    if e.kind == PluginApplyErrorKind::DurableFailure {
                        failure.durable = PluginEnablementDurable::CommitRecoveryDeferred;
                    }
                    failed(
                        failure,
                        PluginEnablementCauseKind::DurableCommit,
                        e,
                        PluginEnablementRecovery::FileRecoveryRequired,
                    )
                })?;
        report.durable = match committed.durability {
            PluginApplyDurability::Durable => PluginEnablementDurable::Durable,
            PluginApplyDurability::CleanupDeferred => PluginEnablementDurable::CleanupDeferred,
            PluginApplyDurability::RecoveryDeferred => {
                PluginEnablementDurable::CommitRecoveryDeferred
            }
        };
        report.committed_revision = committed.committed_revision;
        if report.durable == PluginEnablementDurable::CommitRecoveryDeferred {
            report = failed(
                report,
                PluginEnablementCauseKind::DurableCommit,
                "commit point recovery must complete before runtime apply",
                PluginEnablementRecovery::FileRecoveryRequired,
            );
            return self.publish_enablement_operation(
                report,
                old_status.as_ref().clone(),
                desired_status,
            );
        }
        apply_checkpoint();
        self.apply_committed_enablement(
            &paths,
            session,
            prepared,
            binding,
            committed.receipt,
            report,
            old_status.as_ref().clone(),
            desired_status,
        )
    }

    fn apply_committed_enablement(
        &self,
        paths: &ProjectPaths,
        session: (PathBuf, String, u64),
        prepared: PreparedPluginSelection,
        binding: RuntimeEnablementBinding,
        receipt: PendingPluginApply,
        mut report: PluginEnablementOperationReport,
        old_status: EditorPluginStatusReport,
        desired_status: EditorPluginStatusReport,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        report.phase = PluginEnablementPhase::RuntimeApply;
        if report.durable == PluginEnablementDurable::CleanupDeferred {
            if let Err(error) = recover_plugin_apply_files(paths) {
                report = failed(
                    report,
                    PluginEnablementCauseKind::DurableCommit,
                    error,
                    PluginEnablementRecovery::FileRecoveryRequired,
                );
                return self.publish_enablement_operation(report, old_status, desired_status);
            }
        }
        let ready = (|| -> Result<(), String> {
            let observed =
                verify_pending_plugin_apply(paths, &receipt).map_err(|e| e.to_string())?;
            if Some(&observed.revision) != report.committed_revision.as_ref() {
                return Err("committed manifest revision changed before apply".into());
            }
            if self.active_project_session_focus_target().as_ref() != Some(&session) {
                return Err("project session changed after durable commit".into());
            }
            Ok(())
        })();
        if let Err(error) = ready {
            report = failed(
                report,
                PluginEnablementCauseKind::RevisionChanged,
                error,
                PluginEnablementRecovery::PendingRuntimeApply,
            );
            return self.publish_enablement_operation(report, old_status, desired_status);
        }
        if !prepared.live_supported {
            report = failed(report, PluginEnablementCauseKind::LifecycleApply,
                "the selected Project/native package has no proven live owner; close and resolve explicitly before reopen",
                PluginEnablementRecovery::RuntimeRepairRequired);
            return self.publish_enablement_operation(report, old_status, desired_status);
        }
        let snapshot = match self.apply_bound_plugin_enablement(
            &receipt.plugin_id,
            receipt.enabled,
            &prepared.capabilities,
            prepared.owns_editor_state,
            &binding,
        ) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                report.primary_cause = Some(PluginEnablementCause {
                    kind: error.kind,
                    message: error.primary,
                });
                report.rollback_cause = error.rollback.map(|message| PluginEnablementCause {
                    kind: PluginEnablementCauseKind::Rollback,
                    message,
                });
                report.applied = if error.unknown {
                    PluginEnablementApplied::Unknown
                } else {
                    PluginEnablementApplied::NotApplied
                };
                report.recovery = if error.unknown {
                    PluginEnablementRecovery::RuntimeRepairRequired
                } else {
                    PluginEnablementRecovery::PendingRuntimeApply
                };
                return self.publish_enablement_operation(report, old_status, desired_status);
            }
        };
        report.applied = PluginEnablementApplied::Applied;
        report.detail = Some(EditorPluginEnableReport {
            plugin_id: receipt.plugin_id.clone(),
            enabled: receipt.enabled,
            project_selection: prepared.selection,
            editor_capabilities: prepared.capabilities,
            capability_snapshot: snapshot,
            diagnostics: Vec::new(),
        });
        self.finish_applied_enablement(paths, &receipt, report, old_status, desired_status)
    }
    fn publish_enablement_operation(
        &self,
        report: PluginEnablementOperationReport,
        mut old_status: EditorPluginStatusReport,
        mut desired_status: EditorPluginStatusReport,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let report = self.preserve_plugin_enablement_repair(report);
        let status = if report.applied == PluginEnablementApplied::Applied {
            &mut desired_status
        } else {
            // A new catalog row may be displayed, but manifest intent is never passed off as live.
            if !old_status
                .plugins
                .iter()
                .any(|p| p.plugin_id == report.plugin_id)
            {
                if let Some(mut row) = desired_status
                    .plugins
                    .iter()
                    .find(|p| p.plugin_id == report.plugin_id)
                    .cloned()
                {
                    row.enabled = false;
                    old_status.plugins.push(row);
                }
            }
            &mut old_status
        };
        status.enablement = Some(report.clone());
        // One report, one new Arc, including typed state; ptr_eq projection caches invalidate.
        self.publish_project_plugin_status(status.clone());
        if report.recovery == PluginEnablementRecovery::None {
            Ok(report)
        } else {
            Err(report)
        }
    }

    fn finish_applied_enablement(
        &self,
        paths: &ProjectPaths,
        receipt: &PendingPluginApply,
        mut report: PluginEnablementOperationReport,
        old_status: EditorPluginStatusReport,
        desired_status: EditorPluginStatusReport,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        report.phase = PluginEnablementPhase::Reconcile;
        match acknowledge_plugin_apply(
            paths,
            &receipt.operation_id,
            report
                .committed_revision
                .as_ref()
                .expect("validated committed revision"),
        ) {
            Ok(PluginApplyDurability::Durable) => {
                report.phase = PluginEnablementPhase::Complete;
                report.recovery = PluginEnablementRecovery::None;
            }
            Ok(PluginApplyDurability::CleanupDeferred) => {
                report.durable = PluginEnablementDurable::CleanupDeferred;
                report.phase = PluginEnablementPhase::Complete;
                report.recovery = PluginEnablementRecovery::None;
            }
            Ok(PluginApplyDurability::RecoveryDeferred) => {
                report.recovery = PluginEnablementRecovery::FileRecoveryRequired;
                report.primary_cause = Some(PluginEnablementCause {
                    kind: PluginEnablementCauseKind::Acknowledge,
                    message: "runtime applied; acknowledgement commit point needs file recovery"
                        .into(),
                });
            }
            Err(error) => {
                report.recovery = if error.kind == PluginApplyErrorKind::StaleRevision {
                    PluginEnablementRecovery::RuntimeRepairRequired
                } else {
                    PluginEnablementRecovery::PendingRuntimeApply
                };
                report.primary_cause = Some(PluginEnablementCause {
                    kind: PluginEnablementCauseKind::Acknowledge,
                    message: error.to_string(),
                });
            }
        }
        self.publish_enablement_operation(report, old_status, desired_status)
    }
    fn runtime_enablement_matches(
        &self,
        plugin_id: &str,
        enabled: bool,
        targets: &[String],
    ) -> bool {
        use crate::core::plugin::EditorPluginState;
        let _capability = self.lock_editor_capability_updates();
        let state = self.plugin_manager().state_snapshot();
        let wanted = if enabled {
            EditorPluginState::Active
        } else {
            EditorPluginState::Disabled
        };
        state
            .entry(plugin_id)
            .is_some_and(|entry| entry.state() == wanted)
            && targets
                .iter()
                .all(|target| self.capability_snapshot().is_enabled(target) == enabled)
    }

    fn prepare_bound_plugin_selection(
        &self,
        root: &Path,
        manifest: &ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(PreparedPluginSelection, RuntimeEnablementBinding), String> {
        self.prepare_bound_plugin_selection_with_checkpoint(
            root,
            manifest,
            plugin_id,
            enabled,
            || (),
        )
    }
    fn prepare_bound_plugin_selection_with_checkpoint(
        &self,
        root: &Path,
        manifest: &ProjectManifest,
        plugin_id: &str,
        enabled: bool,
        checkpoint: impl FnOnce(),
    ) -> Result<(PreparedPluginSelection, RuntimeEnablementBinding), String> {
        // Initialize discovery before the baseline; discovery and plan preparation have no live effects.
        let _ = zircon_runtime::plugin::native::discovery::discover_native_plugins(
            self.plugin_directory(root),
        );
        let before = self.capture_enablement_binding(root)?;
        let prepared = self.prepare_durable_plugin_selection(root, manifest, plugin_id, enabled)?;
        checkpoint();
        let after = self.capture_enablement_binding(root)?;
        if before.manager_generation != after.manager_generation
            || before.catalog_generation != after.catalog_generation
            || before.native_generation != after.native_generation
            || before.native_root != after.native_root
            || before.capabilities != after.capabilities
            || before.snapshot != after.snapshot
        {
            return Err("plugin owner generation changed while preparing the durable plan".into());
        }
        Ok((prepared, before))
    }
    /// Explicit retry binds the durable intent to a fresh Ready session and fresh catalogs.
    /// Matching runtime state permits acknowledgement only, without replaying lifecycle hooks.
    pub fn retry_project_plugin_enablement(
        &self,
        root: impl AsRef<Path>,
        operation_id: &str,
        expected_committed_revision: &SourceRevision,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let mut report = operation(root.as_ref(), "", false);
        report.operation_id = operation_id.to_string();
        let _transition = self.begin_project_session_transition().map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::IdentityChanged,
                e,
                PluginEnablementRecovery::PendingRuntimeApply,
            )
        })?;
        let paths = ProjectPaths::from_root(root.as_ref()).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::Prepare,
                e,
                PluginEnablementRecovery::FileRecoveryRequired,
            )
        })?;
        let session = self
            .active_project_session_focus_target()
            .filter(|s| s.0 == paths.root())
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::IdentityChanged,
                    "a Ready session is required",
                    PluginEnablementRecovery::RestartRequired,
                )
            })?;
        if let Some(fence) =
            self.plugin_enablement_repair_for_session(paths.root(), &session.1, session.2)
        {
            return Err(fence);
        }
        recover_plugin_apply_files(&paths).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::DurableCommit,
                e,
                PluginEnablementRecovery::FileRecoveryRequired,
            )
        })?;
        let receipt = pending_plugin_apply(&paths)
            .map_err(|e| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    e,
                    PluginEnablementRecovery::FileRecoveryRequired,
                )
            })?
            .filter(|p| p.operation_id == operation_id)
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::RevisionChanged,
                    "pending operation changed",
                    PluginEnablementRecovery::PendingRuntimeApply,
                )
            })?;
        report.plugin_id = receipt.plugin_id.clone();
        report.requested_enabled = receipt.enabled;
        report.project_root = paths.root().to_path_buf();
        report.project_instance = session.1.clone();
        report.session_generation = session.2;
        report.durable = PluginEnablementDurable::Durable;
        report.committed_revision = Some(expected_committed_revision.clone());
        let document = verify_pending_plugin_apply(&paths, &receipt).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::RevisionChanged,
                e,
                PluginEnablementRecovery::PendingRuntimeApply,
            )
        })?;
        if &document.revision != expected_committed_revision {
            return Err(failed(
                report,
                PluginEnablementCauseKind::RevisionChanged,
                "expected committed revision changed",
                PluginEnablementRecovery::PendingRuntimeApply,
            ));
        }
        let old = self.published_plugin_status_report();
        let (prepared, binding) = self
            .prepare_bound_plugin_selection(
                paths.root(),
                &document.manifest,
                &receipt.plugin_id,
                receipt.enabled,
            )
            .map_err(|e| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    e,
                    PluginEnablementRecovery::PendingRuntimeApply,
                )
            })?;
        let desired = self.native_plugin_status_report_from_load_report(
            &prepared.candidate,
            &prepared.native_report,
        );
        if prepared.live_supported
            && self.runtime_enablement_matches(
                &receipt.plugin_id,
                receipt.enabled,
                &prepared.capabilities,
            )
        {
            report.applied = PluginEnablementApplied::Applied;
            return self.finish_applied_enablement(
                &paths,
                &receipt,
                report,
                old.as_ref().clone(),
                desired,
            );
        }
        self.apply_committed_enablement(
            &paths,
            session,
            prepared,
            binding,
            receipt,
            report,
            old.as_ref().clone(),
            desired,
        )
    }

    #[cfg(test)]
    pub(crate) fn open_enablement_test_project(&self, root: &Path) -> Result<(), String> {
        let intent = self
            .local_open_project_intent(root)
            .map_err(|e| e.to_string())?;
        let preflight = self
            .preflight_existing_project_launch(&intent, root)
            .map_err(|e| e.to_string())?;
        let admission = self
            .session_admission_request(&intent)
            .map_err(|e| e.to_string())?;
        self.open_project_document_with_admission(preflight, &admission)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(test)]
    pub(crate) fn enablement_test_generation(&self) -> u64 {
        self.plugin_manager().state_snapshot().generation()
    }
    #[cfg(test)]
    pub(crate) fn set_plugin_enabled_changing_generation(
        &self,
        root: &Path,
        id: &str,
        enabled: bool,
        after_commit: bool,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let change = || {
            let before = self.enablement_test_generation();
            self.plugin_manager()
                .clear_project_registration_reports()
                .expect("publish a fresh real catalog");
            assert_ne!(
                before,
                self.enablement_test_generation(),
                "checkpoint must publish a real owner generation"
            );
        };
        if after_commit {
            self.commit_plugin_enablement_with_hooks(
                root,
                id,
                enabled,
                None,
                None,
                || (),
                |_| (),
                change,
            )
        } else {
            self.commit_plugin_enablement_with_hooks(
                root,
                id,
                enabled,
                None,
                None,
                change,
                |_| (),
                || (),
            )
        }
    }
    #[cfg(test)]
    pub(crate) fn set_plugin_enabled_with_commit_guard<G>(
        &self,
        root: &Path,
        id: &str,
        enabled: bool,
        guard: impl FnOnce(&ProjectPaths) -> G,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        self.commit_plugin_enablement_with_hooks(root, id, enabled, None, None, || (), guard, || ())
    }
    #[cfg(test)]
    pub(crate) fn register_enablement_test_project_package(
        &self,
        root: &Path,
        id: &str,
    ) -> Result<(), String> {
        let paths = ProjectPaths::from_root(root).map_err(|e| e.to_string())?;
        let document = PluginApplyDocument::read(&paths).map_err(|e| e.to_string())?;
        let native = zircon_runtime::plugin::native::discovery::discover_native_plugins(
            self.plugin_directory(root),
        );
        let mut completed =
            self.complete_project_plugin_manifest_with_native_report(&document.manifest, &native);
        completed
            .plugins
            .selections
            .iter_mut()
            .filter(|p| p.id == id)
            .for_each(|p| p.enabled = true);
        let reports = self.selected_native_editor_plugin_registration_reports_from_load_report(
            &native,
            &completed.plugins,
        );
        self.plugin_manager()
            .publish_project_registration_reports(reports)
            .map_err(|e| e.to_string())?;
        let state = self.plugin_manager().state_snapshot();
        if state.catalog_snapshot().package(id).is_some()
            && state.entry(id).is_some_and(|entry| {
                entry.source() == crate::core::plugin::EditorPluginSource::Project
            })
        {
            Ok(())
        } else {
            Err("fixture did not publish a real Project/native catalog owner".into())
        }
    }

    pub fn revert_project_plugin_enablement(
        &self,
        root: impl AsRef<Path>,
        operation_id: &str,
        expected_committed_revision: &SourceRevision,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let mut report = operation(root.as_ref(), "", false);
        let paths = ProjectPaths::from_root(root.as_ref()).map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::Prepare,
                e,
                PluginEnablementRecovery::PendingRuntimeApply,
            )
        })?;
        let receipt = pending_plugin_apply(&paths)
            .map_err(|e| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    e,
                    PluginEnablementRecovery::FileRecoveryRequired,
                )
            })?
            .filter(|p| p.operation_id == operation_id)
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::RevisionChanged,
                    "pending operation changed",
                    PluginEnablementRecovery::PendingRuntimeApply,
                )
            })?;
        report.plugin_id = receipt.plugin_id.clone();
        if let Some(session) = self
            .active_project_session_focus_target()
            .filter(|s| s.0 == paths.root())
        {
            if let Some(fence) =
                self.plugin_enablement_repair_for_session(paths.root(), &session.1, session.2)
            {
                return Err(fence);
            }
        }
        // This is a new durable command. It never copies an old manifest over a newer revision.
        self.commit_plugin_enablement(
            paths.root(),
            &receipt.plugin_id,
            receipt.previous_enabled,
            Some(expected_committed_revision),
            Some(operation_id),
        )
    }
    pub fn inspect_plugin_enablement_recovery(
        &self,
        root: impl AsRef<Path>,
        admission: Option<PluginEnablementAdmission>,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let report = operation(root.as_ref(), "plugin-enablement", false);
        let _transition = self.begin_project_session_transition().map_err(|e| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::IdentityChanged,
                e,
                PluginEnablementRecovery::FileRecoveryRequired,
            )
        })?;
        let mut report = self.inspect_plugin_enablement_unlocked(root.as_ref());
        if admission.is_some() {
            report.recovery_admission = admission;
        }
        let status = self.published_plugin_status_report();
        self.publish_enablement_operation(report, status.as_ref().clone(), status.as_ref().clone())
    }
    fn inspect_plugin_enablement_unlocked(&self, root: &Path) -> PluginEnablementOperationReport {
        let mut report = operation(root, "plugin-enablement", false);
        report.phase = PluginEnablementPhase::Reconcile;
        let inspected = (|| -> Result<(), PluginApplyError> {
            let paths = ProjectPaths::from_root(root).map_err(|e| PluginApplyError {
                kind: PluginApplyErrorKind::Io,
                cause: e.to_string(),
            })?;
            report.project_root = paths.root().to_path_buf();
            match pending_plugin_apply(&paths)? {
                None => {
                    report.phase = PluginEnablementPhase::Complete;
                    return Ok(());
                }
                Some(receipt) => {
                    report.operation_id = receipt.operation_id.clone();
                    report.plugin_id = receipt.plugin_id.clone();
                    report.requested_enabled = receipt.enabled;
                    report.durable = PluginEnablementDurable::Durable;
                    report.project_instance = receipt.session_instance.clone();
                    report.session_generation = receipt.session_generation;
                    let document = verify_pending_plugin_apply(&paths, &receipt)?;
                    report.committed_revision = Some(document.revision);
                    report.recovery = PluginEnablementRecovery::PendingRuntimeApply;
                }
            }
            Ok(())
        })();
        if let Err(error) = inspected {
            report = failed(
                report,
                PluginEnablementCauseKind::Prepare,
                error,
                PluginEnablementRecovery::FileRecoveryRequired,
            );
        }
        if report.operation_id.is_empty() {
            if let Some(id) =
                ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
                    .allocate()
            {
                report.operation_id = id.nonce().to_string();
            }
        }
        // The same Ready owner remains quarantined even if a display or file observation
        // has disappeared. Neither observation is proof that uncertain callbacks completed.
        if let Some(session) = self
            .active_project_session_focus_target()
            .filter(|s| s.0 == report.project_root)
        {
            if let Some(fence) =
                self.plugin_enablement_repair_for_session(&session.0, &session.1, session.2)
            {
                return fence;
            }
        }
        self.preserve_plugin_enablement_repair(report)
    }
    /// Called only by the already admitted project-transition owner. Observation never applies.
    pub(in crate::ui::host) fn reconcile_pending_plugin_enablement_after_open(&self) {
        let Some((root, _, _)) = self.active_project_session_focus_target() else {
            return;
        };
        let mut report = self.inspect_plugin_enablement_unlocked(&root);
        if report.phase == PluginEnablementPhase::Complete
            && report.recovery == PluginEnablementRecovery::None
        {
            return;
        }
        if report.recovery == PluginEnablementRecovery::PendingRuntimeApply {
            report.recovery = PluginEnablementRecovery::RuntimeRepairRequired;
        }
        let status = self.published_plugin_status_report();
        let _ = self.publish_enablement_operation(
            report,
            status.as_ref().clone(),
            status.as_ref().clone(),
        );
    }
    pub fn execute_plugin_enablement_recovery(
        &self,
        root: impl AsRef<Path>,
        command: PluginEnablementRecoveryCommand,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        match command {
            PluginEnablementRecoveryCommand::Inspect { admission } => {
                self.inspect_plugin_enablement_recovery(root, admission)
            }
            PluginEnablementRecoveryCommand::Retry {
                operation_id,
                expected_revision,
            } => self.retry_project_plugin_enablement(root, &operation_id, &expected_revision),
            PluginEnablementRecoveryCommand::Revert {
                operation_id,
                expected_revision,
                admission,
            } => {
                let ready = self.active_project_session_focus_target().is_some_and(|s| {
                    ProjectPaths::from_root(root.as_ref()).is_ok_and(|paths| s.0 == paths.root())
                });
                if ready {
                    self.revert_project_plugin_enablement(root, &operation_id, &expected_revision)
                } else {
                    self.resolve_plugin_enablement_before_open(
                        root.as_ref(),
                        &operation_id,
                        &expected_revision,
                        admission,
                    )
                }
            }
            PluginEnablementRecoveryCommand::RecoverFiles { admission } => {
                let report = operation(root.as_ref(), "plugin-enablement", false);
                self.with_plugin_enablement_recovery_admission(
                    root.as_ref(),
                    admission.request(),
                    |paths| recover_plugin_apply_files(paths),
                )
                .map_err(|e| admission_failure(report, e))?;
                // A fresh preflight is mandatory. A still-pending operation remains a typed block.
                let preflight = crate::core::project::ProjectAuthority::default()
                    .preflight_project(root.as_ref());
                let result = self.inspect_plugin_enablement_recovery(root, Some(admission));
                match (preflight, result) {
                    (Err(error), Ok(report)) => {
                        let report = failed(
                            report,
                            PluginEnablementCauseKind::Prepare,
                            error,
                            PluginEnablementRecovery::FileRecoveryRequired,
                        );
                        let status = self.published_plugin_status_report();
                        self.publish_enablement_operation(
                            report,
                            status.as_ref().clone(),
                            status.as_ref().clone(),
                        )
                    }
                    (_, result) => result,
                }
            }
        }
    }
    fn resolve_plugin_enablement_before_open(
        &self,
        root: &Path,
        operation_id: &str,
        expected_revision: &SourceRevision,
        admission: Option<PluginEnablementAdmission>,
    ) -> Result<PluginEnablementOperationReport, PluginEnablementOperationReport> {
        let mut report = operation(root, "plugin-enablement", false);
        let admission = admission.ok_or_else(|| {
            failed(
                report.clone(),
                PluginEnablementCauseKind::IdentityChanged,
                "explicit project writer admission is required before open",
                PluginEnablementRecovery::FileRecoveryRequired,
            )
        })?;
        let id = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .ok_or_else(|| {
                failed(
                    report.clone(),
                    PluginEnablementCauseKind::Prepare,
                    "operation identities exhausted",
                    PluginEnablementRecovery::FileRecoveryRequired,
                )
            })?;
        report.operation_id = id.nonce().to_string();
        report.expected_revision = Some(expected_revision.clone());
        report.phase = PluginEnablementPhase::DurableCommit;
        let outcome =
            self.with_plugin_enablement_recovery_admission(root, admission.request(), |paths| {
                report.project_root = paths.root().to_path_buf();
                recover_plugin_apply_files(paths)?;
                let committed = match revert_plugin_apply_before_activation(
                    paths,
                    operation_id,
                    expected_revision,
                    report.operation_id.clone(),
                ) {
                    Ok(committed) => committed,
                    Err(error) => {
                        if error.kind == PluginApplyErrorKind::DurableFailure {
                            report.durable = PluginEnablementDurable::CommitRecoveryDeferred;
                        }
                        return Err(error);
                    }
                };
                report.committed_revision = committed.committed_revision.clone();
                report.plugin_id = committed.receipt.plugin_id.clone();
                report.requested_enabled = committed.receipt.enabled;
                report.durable = match committed.durability {
                    PluginApplyDurability::Durable => PluginEnablementDurable::Durable,
                    PluginApplyDurability::CleanupDeferred => {
                        PluginEnablementDurable::CleanupDeferred
                    }
                    PluginApplyDurability::RecoveryDeferred => {
                        PluginEnablementDurable::CommitRecoveryDeferred
                    }
                };
                if committed.durability == PluginApplyDurability::CleanupDeferred {
                    recover_plugin_apply_files(paths)?;
                }
                Ok(committed)
            });
        let committed = match outcome {
            Ok(committed) => committed,
            Err(error) => {
                report = admission_failure(report, error);
                report.recovery_admission = Some(admission);
                if report.durable == PluginEnablementDurable::NotCommitted {
                    return Err(report);
                }
                let status = self.published_plugin_status_report();
                return self.publish_enablement_operation(
                    report,
                    status.as_ref().clone(),
                    status.as_ref().clone(),
                );
            }
        };
        report.plugin_id = committed.receipt.plugin_id.clone();
        report.requested_enabled = committed.receipt.enabled;
        report.committed_revision = committed.committed_revision.clone();
        report.durable = match committed.durability {
            PluginApplyDurability::Durable => PluginEnablementDurable::Durable,
            PluginApplyDurability::CleanupDeferred => PluginEnablementDurable::CleanupDeferred,
            PluginApplyDurability::RecoveryDeferred => {
                PluginEnablementDurable::CommitRecoveryDeferred
            }
        };
        report.recovery_admission = Some(admission);
        report.recovery = if committed.durability == PluginApplyDurability::RecoveryDeferred {
            PluginEnablementRecovery::FileRecoveryRequired
        } else {
            PluginEnablementRecovery::RestartRequired
        };
        if report.recovery != PluginEnablementRecovery::FileRecoveryRequired {
            if let Err(error) =
                crate::core::project::ProjectAuthority::default().preflight_project(root)
            {
                report = failed(
                    report,
                    PluginEnablementCauseKind::Prepare,
                    error,
                    PluginEnablementRecovery::FileRecoveryRequired,
                );
            } else {
                report.phase = PluginEnablementPhase::Complete;
                // New durable resolution, exact old operation/revision/receipt session only.
                self.clear_plugin_enablement_repair_after_resolution(
                    &report.project_root,
                    operation_id,
                    expected_revision,
                    &committed.receipt,
                );
            }
        }
        let status = self.published_plugin_status_report();
        self.publish_enablement_operation(report, status.as_ref().clone(), status.as_ref().clone())
    }
}
fn admission_failure(
    mut report: PluginEnablementOperationReport,
    failure: PluginEnablementAdmissionFailure,
) -> PluginEnablementOperationReport {
    report = failed(
        report,
        PluginEnablementCauseKind::DurableCommit,
        failure.primary,
        PluginEnablementRecovery::FileRecoveryRequired,
    );
    report.rollback_cause = failure.release.map(|message| PluginEnablementCause {
        kind: PluginEnablementCauseKind::Rollback,
        message,
    });
    report
}
