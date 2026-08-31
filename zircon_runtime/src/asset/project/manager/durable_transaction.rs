use std::io;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::{project::RelPath, resource::ResourceId};

use crate::asset::AssetImportError;
use crate::asset::artifact::IblSourceCubemapStagingStore;
use crate::asset::project::{
    ProjectGenerationPhase, ProjectManifest, ProjectPaths, ResolvedProjectPathIdentity,
};
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::core::resource::io::transaction::{
    DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport, JournalDocument,
    RecoveryPolicy, commit_prepared_files as commit_core_files, recover_pending_transactions,
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
    let mut policy = ProjectRecoveryPolicy::new(paths, &manifest.asset_roots)?;
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
    artifact_root: ResolvedProjectPathIdentity,
    registry_root: ResolvedProjectPathIdentity,
    registry_path: ResolvedProjectPathIdentity,
    asset_roots: Vec<ResolvedProjectPathIdentity>,
    ibl_bundle_store: IblSourceCubemapStagingStore,
}

impl ProjectRecoveryPolicy {
    fn new(paths: &ProjectPaths, roots: &[RelPath]) -> io::Result<Self> {
        let cache_root = ProjectPaths::resolve_identity(paths.cache_root())?;
        Ok(Self {
            artifact_root: ProjectPaths::resolve_identity(paths.asset_artifact_root())?,
            registry_root: ProjectPaths::resolve_identity(paths.registry_root())?,
            registry_path: ProjectPaths::resolve_identity(
                paths.registry_root().join(ASSET_REGISTRY_FILE),
            )?,
            asset_roots: roots
                .iter()
                .map(|root| ProjectPaths::resolve_identity(paths.asset_root(root)))
                .collect::<io::Result<Vec<_>>>()?,
            ibl_bundle_store: IblSourceCubemapStagingStore::new(
                cache_root.operation_path().to_path_buf(),
            ),
        })
    }
}

impl RecoveryPolicy for ProjectRecoveryPolicy {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        let target = recovery_identity(document.target())?;
        let retired_paths = document.retired_paths().collect::<Vec<_>>();
        if !retired_paths.is_empty() {
            if retired_paths.len() == 1 {
                let retired_path = retired_paths[0];
                let retired = recovery_identity(retired_path)?;
                if self.is_relocatable_project_entry(document.target(), &target)?
                    && self.is_relocatable_project_entry(retired_path, &retired)?
                    && self.is_meta_path(document.target()) == self.is_meta_path(retired_path)
                {
                    return Ok(());
                }
            }
            if self.is_registry_entry(document.target(), &target)?
                && self.is_asset_source_retirement_set(&retired_paths)?
            {
                return Ok(());
            }
            return Err(format!(
                "project generation retirement is outside the source mutation set: {}",
                document.target().display()
            ));
        }
        if self.is_registry_entry(document.target(), &target)?
            || self.is_artifact_manifest_entry(document.target(), &target)?
        {
            return Ok(());
        }
        if self
            .ibl_bundle_store
            .validate_bundle_target(target.operation_path())
            .is_ok()
        {
            return Ok(());
        }
        if self.is_import_source_plan_entry(document.target(), &target)? {
            return Ok(());
        }
        if self.is_meta_path(document.target())
            && self.is_relocatable_project_entry(document.target(), &target)?
        {
            return Ok(());
        }
        Err(format!(
            "project generation target is outside the durable publication set: {}",
            document.target().display()
        ))
    }
}

impl ProjectRecoveryPolicy {
    fn is_relocatable_project_file(&self, path: &ResolvedProjectPathIdentity) -> bool {
        self.asset_roots.iter().any(|root| path.is_within(root))
    }

    fn is_relocatable_project_entry(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        if !self.is_relocatable_project_file(target) {
            return Ok(false);
        }
        let parent = recovery_parent_identity(raw_path)?;
        Ok(self.asset_roots.iter().any(|root| parent.is_within(root)))
    }

    fn is_registry_entry(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        if target != &self.registry_path
            || raw_path.file_name() != Some(std::ffi::OsStr::new(ASSET_REGISTRY_FILE))
        {
            return Ok(false);
        }
        Ok(recovery_parent_identity(raw_path)? == self.registry_root)
    }

    fn is_artifact_manifest_entry(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        if !self.is_artifact_manifest(target) {
            return Ok(false);
        }
        let parent = recovery_parent_identity(raw_path)?;
        let Some(mut relative) = parent.relative_to(&self.artifact_root) else {
            return Ok(false);
        };
        let Some(file_name) = raw_path.file_name() else {
            return Ok(false);
        };
        relative.push(file_name);
        Ok(Self::is_artifact_manifest_relative(&relative))
    }

    fn is_meta_path(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".zmeta"))
    }

    fn is_asset_source_retirement_set(&self, paths: &[&Path]) -> Result<bool, String> {
        if paths.len() != 2 {
            return Ok(false);
        }
        let (source, meta) = if self.is_meta_path(paths[0]) && !self.is_meta_path(paths[1]) {
            (paths[1], paths[0])
        } else if !self.is_meta_path(paths[0]) && self.is_meta_path(paths[1]) {
            (paths[0], paths[1])
        } else {
            return Ok(false);
        };
        let source_identity = recovery_identity(source)?;
        let meta_identity = recovery_identity(meta)?;
        if !self.is_relocatable_project_entry(source, &source_identity)?
            || !self.is_relocatable_project_entry(meta, &meta_identity)?
        {
            return Ok(false);
        }
        Ok(super::meta_path_for_source::meta_path_for_source(source) == meta)
    }

    /// External model import plans may stage only one direct source file in `models/` plus an
    /// optional OBJ material companion. Recovery never widens this to arbitrary asset-root writes.
    fn is_import_source_plan_target(&self, path: &ResolvedProjectPathIdentity) -> bool {
        self.asset_roots.iter().any(|root| {
            let Some(relative) = path.relative_to(root) else {
                return false;
            };
            Self::is_import_source_plan_relative(&relative)
        })
    }

    fn is_import_source_plan_entry(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        if !self.is_import_source_plan_target(target) {
            return Ok(false);
        }
        let parent = recovery_parent_identity(raw_path)?;
        let Some(file_name) = raw_path.file_name() else {
            return Ok(false);
        };
        Ok(self.asset_roots.iter().any(|root| {
            let Some(mut relative) = parent.relative_to(root) else {
                return false;
            };
            relative.push(file_name);
            Self::is_import_source_plan_relative(&relative)
        }))
    }

    fn is_import_source_plan_relative(relative: &Path) -> bool {
        let mut components = relative.components();
        let Some(directory) = components.next() else {
            return false;
        };
        let Some(file_name) = components.next() else {
            return false;
        };
        if components.next().is_some()
            || !directory
                .as_os_str()
                .to_str()
                .is_some_and(|directory| directory.eq_ignore_ascii_case("models"))
            || file_name.as_os_str().is_empty()
        {
            return false;
        }
        let extension = relative
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase);
        matches!(extension.as_deref(), Some("obj" | "glb" | "mtl"))
    }

    fn is_artifact_manifest(&self, target: &ResolvedProjectPathIdentity) -> bool {
        let Some(relative) = target.relative_to(&self.artifact_root) else {
            return false;
        };
        Self::is_artifact_manifest_relative(&relative)
    }

    fn is_artifact_manifest_relative(relative: &Path) -> bool {
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
        relative.extension().and_then(|value| value.to_str()) == Some("zasset")
            && relative
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.parse::<ResourceId>().is_ok())
    }
}

fn recovery_identity(path: &Path) -> Result<ResolvedProjectPathIdentity, String> {
    ProjectPaths::resolve_identity(path).map_err(|error| {
        format!(
            "project recovery could not resolve target identity {}: {error}",
            ProjectPaths::display_path(path).display()
        )
    })
}

fn recovery_parent_identity(path: &Path) -> Result<ResolvedProjectPathIdentity, String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "project recovery target has no parent directory: {}",
            ProjectPaths::display_path(path).display()
        )
    })?;
    ProjectPaths::resolve_identity(parent).map_err(|error| {
        format!(
            "project recovery could not resolve original parent identity {}: {error}",
            ProjectPaths::display_path(parent).display()
        )
    })
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
    let root = ResolvedProjectPathIdentity::from(ProjectPaths::resolve_existing(project_root)?);
    let derived = ResolvedProjectPathIdentity::from(ProjectPaths::resolve_existing(derived)?);
    if !derived.is_within(&root) {
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
        ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
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

#[cfg(test)]
mod recovery_policy_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime_interface::{project::RelPath, resource::ResourceId};

    use super::{ProjectPaths, ProjectRecoveryPolicy};

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn physical_asset_target_does_not_authorize_an_outside_raw_directory_entry() {
        let fixture = unique_test_root("raw-entry-containment");
        let project_root = fixture.join("project");
        let outside_root = fixture.join("outside");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&outside_root).unwrap();
        let asset_root = RelPath::parse("assets").unwrap();
        let paths = ProjectPaths::from_root(&project_root).unwrap();
        paths
            .ensure_layout(std::slice::from_ref(&asset_root))
            .unwrap();
        let policy = ProjectRecoveryPolicy::new(&paths, &[asset_root]).unwrap();
        let physical_target =
            ProjectPaths::resolve_identity(project_root.join("assets/panel.zui.zmeta")).unwrap();
        let outside_entry = outside_root.join("panel.zui.zmeta");

        assert!(policy.is_relocatable_project_file(&physical_target));
        assert!(
            !policy
                .is_relocatable_project_entry(&outside_entry, &physical_target)
                .unwrap()
        );

        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn canonical_publication_target_does_not_authorize_a_different_raw_leaf_layout() {
        let fixture = unique_test_root("raw-leaf-layout");
        let project_root = fixture.join("project");
        fs::create_dir_all(&project_root).unwrap();
        let asset_root = RelPath::parse("assets").unwrap();
        let paths = ProjectPaths::from_root(&project_root).unwrap();
        paths
            .ensure_layout(std::slice::from_ref(&asset_root))
            .unwrap();
        let policy = ProjectRecoveryPolicy::new(&paths, &[asset_root]).unwrap();

        let import_target =
            ProjectPaths::resolve_identity(project_root.join("assets/models/source.obj")).unwrap();
        assert!(policy.is_import_source_plan_target(&import_target));
        assert!(
            !policy
                .is_import_source_plan_entry(
                    &project_root.join("assets/arbitrary.txt"),
                    &import_target,
                )
                .unwrap()
        );

        let artifact_name = format!(
            "{}.zasset",
            ResourceId::from_stable_label("raw-leaf-layout-artifact")
        );
        let artifact_target = ProjectPaths::resolve_identity(
            paths
                .asset_artifact_root()
                .join("models")
                .join(artifact_name),
        )
        .unwrap();
        assert!(policy.is_artifact_manifest(&artifact_target));
        assert!(
            !policy
                .is_artifact_manifest_entry(
                    &paths.asset_artifact_root().join("models/arbitrary.cache"),
                    &artifact_target,
                )
                .unwrap()
        );

        fs::remove_dir_all(fixture).unwrap();
    }

    fn unique_test_root(label: &str) -> PathBuf {
        let root = test_output_root().join(format!(
            "zircon-project-recovery-{label}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn test_output_root() -> PathBuf {
        std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .expect("resolve current workspace for project recovery test output")
                    .join("target")
            })
            .join("zircon-test-output")
    }
}
