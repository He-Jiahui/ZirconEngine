//! Durable publication of a Scene document and its opaque editor workspace companion.
//!
//! The editor owns the workspace schema. Runtime only validates the physical owner, project
//! generation, and bounded publication paths before handing both bytes to the existing resource
//! transaction substrate. Recovery runs before a project generation is admitted to the registry.

use std::io;
use std::path::{Path, PathBuf};

use crate::asset::project::{ProjectManifest, ProjectPaths, ResolvedProjectPathIdentity};
use crate::asset::{AssetImportError, AssetUri};
use crate::core::resource::io::transaction::{
    commit_prepared_files as commit_core_files, recover_pending_transactions,
    DurableCommitDisposition, DurableCommitReport, JournalDocument, RecoveryPolicy,
};
use crate::scene::world::SceneProjectError;
use crate::scene::Scene;

use super::durable_transaction::{
    record_commit_report, record_recovery_report, recovery_identity, recovery_parent_identity,
    transaction_error, validate_journal_owner, PreparedFileWrite, ProjectTransactionFault,
};
use super::ProjectManager;

const JOURNAL_DIRECTORY: &str = "editor-document";
const TRANSACTION_TAG: &str = "editor-document";
const WORKSPACE_FILE: &str = "editor-workspace.json";

/// Physical project identity and catalog generation admitted by one editor-document operation.
///
/// The workspace payload is intentionally opaque: the editor remains the owner of its schema and
/// passes encoded bytes through this narrow runtime boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorDocumentScope {
    project_root: PathBuf,
    project_generation: u64,
}

impl EditorDocumentScope {
    pub fn new(project_root: impl Into<PathBuf>, project_generation: u64) -> Self {
        Self {
            project_root: project_root.into(),
            project_generation,
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn project_generation(&self) -> u64 {
        self.project_generation
    }
}

#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorDocumentCommitOutcome {
    Durable,
    RecoveryDeferred { journal_directory: PathBuf },
}

/// Controlled publication interruption used by editor integration fixtures to exercise recovery
/// through the real ProjectManager reopen path. The workspace bytes remain opaque to runtime.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorDocumentCommitFault {
    None,
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterTargetReplace(usize),
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterRetiredDelete(usize),
}

#[cfg(any(test, feature = "test-support"))]
impl EditorDocumentCommitFault {
    fn into_runtime_fault(self) -> ProjectTransactionFault {
        match self {
            Self::None => ProjectTransactionFault::None,
            Self::CrashAfterTargetReplace(index) => {
                ProjectTransactionFault::CrashAfterTargetReplace(index)
            }
            Self::CrashAfterRetiredDelete(index) => {
                ProjectTransactionFault::CrashAfterRetiredDelete(index)
            }
        }
    }
}

impl EditorDocumentCommitOutcome {
    pub fn ensure_durable(self) -> Result<(), SceneProjectError> {
        match self {
            Self::Durable => Ok(()),
            Self::RecoveryDeferred { journal_directory } => Err(SceneProjectError::Io(
                io::Error::other(format!(
                    "editor document publication is awaiting recovery at {}; reopen the project before continuing",
                    journal_directory.display()
                )),
            )),
        }
    }
}

pub struct ScopedRuntimeProjectManager<'project> {
    project: &'project ProjectManager,
    project_identity: ResolvedProjectPathIdentity,
    project_generation: u64,
}

impl ProjectManager {
    /// Binds a durable Scene/workspace operation to one physical project and catalog generation.
    pub fn scoped_editor_document(
        &self,
        scope: &EditorDocumentScope,
    ) -> Result<ScopedRuntimeProjectManager<'_>, AssetImportError> {
        let project_identity = ProjectPaths::resolve_identity(self.paths.root())?;
        let requested_identity = ProjectPaths::resolve_identity(scope.project_root())?;
        if project_identity != requested_identity {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "editor document scope project root {} does not match opened project {}",
                    ProjectPaths::display_path(scope.project_root()).display(),
                    ProjectPaths::display_path(self.paths.root()).display()
                ),
            )
            .into());
        }
        let project_generation = self.catalog_input_generation.sequence();
        if project_generation != scope.project_generation() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "editor document scope generation {} is stale; opened project is at generation {}",
                    scope.project_generation(),
                    project_generation
                ),
            )
            .into());
        }
        Ok(ScopedRuntimeProjectManager {
            project: self,
            project_identity,
            project_generation,
        })
    }
}

impl<'project> ScopedRuntimeProjectManager<'project> {
    pub fn project_generation(&self) -> u64 {
        self.project_generation
    }

    pub fn commit_scene_workspace(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        workspace_bytes: &[u8],
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(
            scene_uri,
            scene,
            Some(workspace_bytes),
            ProjectTransactionFault::None,
        )
    }

    /// Publishes a Scene while explicitly retiring the editor workspace document.
    ///
    /// The public `None` workspace save semantics mean that no workspace document remains. The
    /// retirement is staged in the same journal as the Scene write so a directory reparse or a
    /// crash cannot route the Scene outside the project or leave a split pair.
    pub fn commit_scene_without_workspace(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(
            scene_uri,
            scene,
            None,
            ProjectTransactionFault::None,
        )
    }

    /// Test-only fault seam for the editor's real encoded Scene/workspace reopen fixture.
    #[doc(hidden)]
    #[cfg(any(test, feature = "test-support"))]
    pub fn commit_scene_workspace_with_fault_for_test(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        workspace_bytes: &[u8],
        fault: EditorDocumentCommitFault,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(
            scene_uri,
            scene,
            Some(workspace_bytes),
            fault.into_runtime_fault(),
        )
    }

    /// Test-only fault seam for the public `None` workspace retirement semantics.
    #[doc(hidden)]
    #[cfg(any(test, feature = "test-support"))]
    pub fn commit_scene_without_workspace_with_fault_for_test(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        fault: EditorDocumentCommitFault,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(scene_uri, scene, None, fault.into_runtime_fault())
    }

    fn commit_scene_workspace_with_fault(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        workspace_bytes: Option<&[u8]>,
        fault: ProjectTransactionFault,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.ensure_generation()?;
        let scene_path = self
            .project
            .existing_or_primary_project_source_path_for_uri(scene_uri)?;
        self.validate_scene_target(&scene_path)?;
        let workspace_path = self.workspace_path();
        self.validate_workspace_target(&workspace_path)?;

        let scene_asset = scene.to_scene_asset(self.project)?;
        let scene_document = scene_asset
            .to_project_toml_string(|reference| self.project.persist_runtime_reference(reference))
            .map_err(SceneProjectError::from)?;

        let _generation =
            crate::asset::project::lock_project_generation(self.project.paths.root())?;
        self.ensure_generation()?;
        let journal_directory = self.journal_directory();
        validate_journal_owner(&journal_directory)?;
        let scene_write = PreparedFileWrite::new(scene_path, scene_document.into_bytes());
        let writes = match workspace_bytes {
            Some(workspace_bytes) => vec![
                scene_write,
                PreparedFileWrite::new(workspace_path, workspace_bytes.to_vec()),
            ],
            None => {
                let workspace_present = match std::fs::symlink_metadata(&workspace_path) {
                    Ok(_) => true,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => false,
                    Err(error) => return Err(SceneProjectError::Io(error)),
                };
                if workspace_present {
                    vec![scene_write.retiring(workspace_path)]
                } else {
                    vec![scene_write]
                }
            }
        };
        let mut report = DurableCommitReport::default();
        let disposition = commit_core_files(
            &journal_directory,
            TRANSACTION_TAG,
            writes,
            fault,
            &mut report,
        )
        .map_err(transaction_error)?;
        record_commit_report(report);
        Ok(match disposition {
            DurableCommitDisposition::CommitRecoveryDeferred => {
                EditorDocumentCommitOutcome::RecoveryDeferred { journal_directory }
            }
            DurableCommitDisposition::Durable | DurableCommitDisposition::CleanupDeferred => {
                EditorDocumentCommitOutcome::Durable
            }
        })
    }

    fn ensure_generation(&self) -> Result<(), SceneProjectError> {
        let current = self.project.catalog_input_generation().sequence();
        if current != self.project_generation {
            return Err(SceneProjectError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "editor document scope generation {} is stale; current project generation is {}",
                    self.project_generation, current
                ),
            )));
        }
        Ok(())
    }

    fn journal_directory(&self) -> PathBuf {
        self.project.paths.derived_root().join(JOURNAL_DIRECTORY)
    }

    fn workspace_path(&self) -> PathBuf {
        self.project.paths.derived_root().join(WORKSPACE_FILE)
    }

    fn validate_scene_target(&self, path: &Path) -> Result<(), SceneProjectError> {
        let target = ProjectPaths::resolve_identity(path)?;
        let file_name = path.file_name().and_then(|name| name.to_str());
        if !file_name.is_some_and(|name| name.to_ascii_lowercase().ends_with(".scene.toml")) {
            return Err(SceneProjectError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "editor document target is not a Scene source: {}",
                    path.display()
                ),
            )));
        }
        let parent = path.parent().ok_or_else(|| {
            SceneProjectError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "editor document Scene target has no parent directory",
            ))
        })?;
        let parent = ProjectPaths::resolve_identity(parent)?;
        let inside_registered_root = self
            .project
            .project_asset_roots()
            .iter()
            .filter_map(|root| ProjectPaths::resolve_identity(root).ok())
            .any(|root| target.is_within(&root) && parent.is_within(&root));
        if !inside_registered_root {
            return Err(SceneProjectError::Asset(
                AssetImportError::SourceOutsideProjectAssetRoots {
                    path: path.to_path_buf(),
                },
            ));
        }
        Ok(())
    }

    fn validate_workspace_target(&self, path: &Path) -> Result<(), SceneProjectError> {
        let target = ProjectPaths::resolve_identity(path)?;
        let parent = path.parent().ok_or_else(|| {
            SceneProjectError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "editor workspace target has no parent directory",
            ))
        })?;
        let parent = ProjectPaths::resolve_identity(parent)?;
        let expected_parent = ProjectPaths::resolve_identity(self.project.paths.derived_root())?;
        let expected_target = ProjectPaths::resolve_identity(
            self.project_identity
                .operation_path()
                .join(".zircon")
                .join(WORKSPACE_FILE),
        )?;
        if target != expected_target
            || parent != expected_parent
            || path.file_name() != Some(std::ffi::OsStr::new(WORKSPACE_FILE))
        {
            return Err(SceneProjectError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "editor workspace target is outside the project .zircon owner: {}",
                    path.display()
                ),
            )));
        }
        Ok(())
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn commit_scene_workspace_with_test_fault(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        workspace_bytes: &[u8],
        fault: ProjectTransactionFault,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(scene_uri, scene, Some(workspace_bytes), fault)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn commit_scene_without_workspace_with_test_fault(
        &self,
        scene_uri: &AssetUri,
        scene: &Scene,
        fault: ProjectTransactionFault,
    ) -> Result<EditorDocumentCommitOutcome, SceneProjectError> {
        self.commit_scene_workspace_with_fault(scene_uri, scene, None, fault)
    }
}

pub(super) fn recover_editor_document(
    paths: &ProjectPaths,
    manifest: &ProjectManifest,
) -> Result<(), AssetImportError> {
    let directory = paths.derived_root().join(JOURNAL_DIRECTORY);
    if !directory.exists() {
        return Ok(());
    }
    validate_journal_owner(&directory)?;
    let mut policy = EditorDocumentRecoveryPolicy::new(paths, manifest)?;
    let report = recover_pending_transactions(&directory, TRANSACTION_TAG, &mut policy)
        .map_err(transaction_error)?;
    record_recovery_report(report);
    Ok(())
}

struct EditorDocumentRecoveryPolicy {
    workspace_parent: ResolvedProjectPathIdentity,
    workspace_path: ResolvedProjectPathIdentity,
    asset_roots: Vec<ResolvedProjectPathIdentity>,
}

impl EditorDocumentRecoveryPolicy {
    fn new(paths: &ProjectPaths, manifest: &ProjectManifest) -> Result<Self, AssetImportError> {
        Ok(Self {
            workspace_parent: ProjectPaths::resolve_identity(paths.derived_root())?,
            workspace_path: ProjectPaths::resolve_identity(
                paths.derived_root().join(WORKSPACE_FILE),
            )?,
            asset_roots: manifest
                .asset_roots
                .iter()
                .map(|root| ProjectPaths::resolve_identity(paths.asset_root(root)))
                .collect::<io::Result<Vec<_>>>()?,
        })
    }

    fn scene_target_allowed(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        let Some(name) = raw_path.file_name().and_then(|name| name.to_str()) else {
            return Ok(false);
        };
        if !name.to_ascii_lowercase().ends_with(".scene.toml") {
            return Ok(false);
        }
        let parent = recovery_parent_identity(raw_path)?;
        Ok(self
            .asset_roots
            .iter()
            .any(|root| target.is_within(root) && parent.is_within(root)))
    }

    fn workspace_target_allowed(
        &self,
        raw_path: &Path,
        target: &ResolvedProjectPathIdentity,
    ) -> Result<bool, String> {
        Ok(target == &self.workspace_path
            && raw_path.file_name() == Some(std::ffi::OsStr::new(WORKSPACE_FILE))
            && recovery_parent_identity(raw_path)? == self.workspace_parent)
    }
}

impl RecoveryPolicy for EditorDocumentRecoveryPolicy {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        let target = recovery_identity(document.target())?;
        let retired_paths = document.retired_paths().collect::<Vec<_>>();
        if retired_paths.is_empty() && self.workspace_target_allowed(document.target(), &target)? {
            return Ok(());
        }
        if retired_paths.len() == 1
            && self.scene_target_allowed(document.target(), &target)?
            && self
                .workspace_target_allowed(retired_paths[0], &recovery_identity(retired_paths[0])?)?
        {
            return Ok(());
        }
        if retired_paths.is_empty() && self.scene_target_allowed(document.target(), &target)? {
            return Ok(());
        }
        Err(format!(
            "editor document target is outside the Scene/workspace publication set: {}",
            document.target().display()
        ))
    }
}

#[cfg(test)]
#[path = "tests/editor_document_transaction.rs"]
mod tests;
