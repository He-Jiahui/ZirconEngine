use std::io;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::{project::RelPath, resource::ResourceId};

use crate::asset::project::{ProjectGenerationPhase, ProjectManifest, ProjectPaths};
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::AssetImportError;
use crate::core::resource::io::transaction::{
    commit_prepared_files as commit_core_files, recover_pending_transactions,
    DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport, JournalDocument,
    RecoveryPolicy,
};

pub(super) use crate::core::resource::io::transaction::{
    PreparedFileWrite, TransactionFault as ProjectTransactionFault,
};

const JOURNAL_DIRECTORY: &str = "project-generation";
const TRANSACTION_TAG: &str = "project";
const ASSET_REGISTRY_FILE: &str = "asset-registry.json";

#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectFileCommitOutcome {
    Durable,
    RecoveryDeferred { journal_directory: PathBuf },
}

impl ProjectFileCommitOutcome {
    pub(crate) fn ensure_durable(self) -> Result<(), AssetImportError> {
        match self {
            Self::Durable => Ok(()),
            Self::RecoveryDeferred { journal_directory } => Err(io::Error::other(format!(
                "project generation commit marker durability is unresolved at {}; restart or reopen the project to recover the pending transaction",
                journal_directory.display()
            ))
            .into()),
        }
    }
}

pub(super) fn journal_directory(paths: &ProjectPaths) -> PathBuf {
    paths.derived_root().join(JOURNAL_DIRECTORY)
}

pub(super) fn commit_prepared_files(
    journal_directory: &Path,
    writes: Vec<PreparedFileWrite>,
    fault: ProjectTransactionFault,
) -> Result<ProjectFileCommitOutcome, AssetImportError> {
    validate_journal_owner(journal_directory)?;
    let mut report = DurableCommitReport::default();
    let result = commit_core_files(
        journal_directory,
        TRANSACTION_TAG,
        writes,
        fault,
        &mut report,
    );
    record_commit_report(report);
    result
        .map_err(transaction_error)
        .map(|disposition| match disposition {
            DurableCommitDisposition::CommitRecoveryDeferred => {
                ProjectFileCommitOutcome::RecoveryDeferred {
                    journal_directory: journal_directory.to_path_buf(),
                }
            }
            DurableCommitDisposition::Durable | DurableCommitDisposition::CleanupDeferred => {
                ProjectFileCommitOutcome::Durable
            }
        })
}

pub(super) fn recover_project_generation(
    paths: &ProjectPaths,
    manifest: &ProjectManifest,
) -> Result<(), AssetImportError> {
    let directory = journal_directory(paths);
    if !directory.exists() {
        return Ok(());
    }
    let _phase = ProjectGenerationPhase::Recovery.enter();
    validate_journal_owner(&directory)?;
    let mut policy = ProjectRecoveryPolicy::new(paths, &manifest.asset_roots);
    let report = recover_pending_transactions(&directory, TRANSACTION_TAG, &mut policy)
        .map_err(transaction_error)?;
    record_recovery_report(report);
    Ok(())
}

fn record_recovery_report(report: DurableRecoveryReport) {
    #[cfg(feature = "profiling")]
    {
        use crate::core::runtime::diagnostics::profiling::{capture_active, record_counter_batch};

        if !capture_active() {
            return;
        }
        record_counter_batch(
            "resource",
            &[
                (
                    "resource.transaction.recovery_rollback_count",
                    usize_to_f64(report.rollback_count()),
                ),
                (
                    "resource.transaction.recovery_cleanup_count",
                    usize_to_f64(report.cleanup_count()),
                ),
                (
                    "resource.transaction.intent_orphan_cleanup_count",
                    usize_to_f64(report.intent_orphan_cleanup_count()),
                ),
            ],
        );
    }
    #[cfg(not(feature = "profiling"))]
    let _ = report;
}

fn record_commit_report(report: DurableCommitReport) {
    #[cfg(feature = "profiling")]
    {
        use crate::core::runtime::diagnostics::profiling::{capture_active, record_counter_batch};

        if !report.has_commit_activity() || !capture_active() {
            return;
        }
        record_counter_batch(
            "resource",
            &[
                (
                    "resource.transaction.live_rollback_restore_attempt_count",
                    usize_to_f64(report.rollback_restore_attempt_count()),
                ),
                (
                    "resource.transaction.live_rollback_restore_success_count",
                    usize_to_f64(report.rollback_restore_success_count()),
                ),
                (
                    "resource.transaction.deferred_commit_recovery_count",
                    usize_to_f64(report.deferred_commit_recovery_count()),
                ),
                (
                    "resource.transaction.deferred_cleanup_count",
                    usize_to_f64(report.deferred_cleanup_count()),
                ),
            ],
        );
    }
    #[cfg(not(feature = "profiling"))]
    let _ = report;
}

#[cfg(feature = "profiling")]
fn usize_to_f64(value: usize) -> f64 {
    u64::try_from(value).unwrap_or(u64::MAX) as f64
}

fn transaction_error(error: impl std::error::Error + Send + Sync + 'static) -> AssetImportError {
    io::Error::other(error).into()
}

struct ProjectRecoveryPolicy {
    artifact_root: PathBuf,
    registry_path: PathBuf,
    asset_roots: Vec<PathBuf>,
}

impl ProjectRecoveryPolicy {
    fn new(paths: &ProjectPaths, roots: &[RelPath]) -> Self {
        Self {
            artifact_root: physical_key(paths.asset_artifact_root()),
            registry_path: physical_key(paths.registry_root().join(ASSET_REGISTRY_FILE)),
            asset_roots: roots
                .iter()
                .map(|root| physical_key(paths.asset_root(root)))
                .collect(),
        }
    }
}

impl RecoveryPolicy for ProjectRecoveryPolicy {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        if document.retired_path().is_some() {
            return Err("project generation transactions cannot retire live files".to_owned());
        }
        let target = physical_key(document.target());
        if target == self.registry_path || self.is_artifact_manifest(&target) {
            return Ok(());
        }
        let is_meta = document
            .target()
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".zmeta"));
        if is_meta && self.asset_roots.iter().any(|root| target.starts_with(root)) {
            return Ok(());
        }
        Err(format!(
            "project generation target is outside the durable publication set: {}",
            document.target().display()
        ))
    }
}

impl ProjectRecoveryPolicy {
    fn is_artifact_manifest(&self, target: &Path) -> bool {
        let Ok(relative) = target.strip_prefix(&self.artifact_root) else {
            return false;
        };
        let mut components = relative.components();
        let Some(namespace) = components
            .next()
            .and_then(|component| component.as_os_str().to_str())
        else {
            return false;
        };
        if components.next().is_none()
            || namespace.eq_ignore_ascii_case("chunks")
            || namespace.eq_ignore_ascii_case(".staging")
        {
            return false;
        }
        target.extension().and_then(|value| value.to_str()) == Some("zasset")
            && target
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.parse::<ResourceId>().is_ok())
    }
}

fn physical_key(path: impl AsRef<Path>) -> PathBuf {
    ProjectPaths::filesystem_identity_key(path)
}

fn validate_journal_owner(directory: &Path) -> Result<(), AssetImportError> {
    let derived = directory.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "project generation journal has no derived-state owner",
        )
    })?;
    let project_root = derived.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "project generation journal has no project owner",
        )
    })?;
    validate_real_directory(project_root)?;
    validate_real_directory(derived)?;
    if directory.exists() {
        validate_real_directory(directory)?;
    }
    let root = ProjectPaths::resolve_existing_path(project_root)?;
    let derived = ProjectPaths::resolve_existing_path(derived)?;
    if !derived.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "project derived-state owner escapes the project root",
        )
        .into());
    }
    Ok(())
}

fn validate_real_directory(path: &Path) -> Result<(), AssetImportError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if is_link_or_reparse(&metadata) || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "project transaction owner must be a real directory: {}",
                path.display()
            ),
        )
        .into());
    }
    Ok(())
}

#[cfg(all(test, feature = "profiling"))]
mod tests {
    use crate::core::resource::io::transaction::{DurableCommitReport, DurableRecoveryReport};
    use crate::core::runtime::diagnostics::profiling::{
        reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
    };

    use super::{record_commit_report, record_recovery_report};

    #[test]
    fn project_adapter_publishes_resource_neutral_live_rollback_report() {
        let _guard = test_capture_lock();
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "project-durable-live-rollback-report".to_owned();
        config.max_counters = 8;
        start_capture(config);

        record_commit_report(DurableCommitReport::from_activity_counts(3, 2, 1, 1));

        let snapshot = snapshot();
        reset_capture();
        let values = snapshot
            .counters
            .iter()
            .map(|counter| (counter.name.as_str(), counter.value))
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(
            values["resource.transaction.live_rollback_restore_attempt_count"],
            3.0
        );
        assert_eq!(
            values["resource.transaction.live_rollback_restore_success_count"],
            2.0
        );
        assert_eq!(
            values["resource.transaction.deferred_commit_recovery_count"],
            1.0
        );
        assert_eq!(values["resource.transaction.deferred_cleanup_count"], 1.0);
    }

    #[test]
    fn project_adapter_publishes_resource_neutral_recovery_report() {
        let _guard = test_capture_lock();
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "project-durable-recovery-report".to_owned();
        config.max_counters = 8;
        start_capture(config);

        record_recovery_report(DurableRecoveryReport::new(2, 3, 4));

        let snapshot = snapshot();
        reset_capture();
        let values = snapshot
            .counters
            .iter()
            .map(|counter| (counter.name.as_str(), counter.value))
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(values["resource.transaction.recovery_rollback_count"], 2.0);
        assert_eq!(values["resource.transaction.recovery_cleanup_count"], 3.0);
        assert_eq!(
            values["resource.transaction.intent_orphan_cleanup_count"],
            4.0
        );
    }
}
