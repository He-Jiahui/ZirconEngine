use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::asset::project::{ProjectManifestError, ProjectPaths};
use zircon_runtime::asset::AssetImportError;
use zircon_runtime::core::CoreError;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime_interface::project::{
    CanonicalDescriptorIdentityError, ProjectNameError, ProjectTemplatePackError,
};

#[derive(Debug, Error)]
pub enum ProjectAuthorityError {
    #[error("project name is invalid: {source}")]
    ProjectName {
        #[from]
        #[source]
        source: ProjectNameError,
    },
    #[error("canonical project descriptor identity is invalid: {source}")]
    CanonicalDescriptorIdentity {
        #[from]
        #[source]
        source: CanonicalDescriptorIdentityError,
    },
    #[error("current project manifest preflight is missing its required project GUID")]
    CurrentManifestMissingProjectGuid,
    #[error("project location cannot be empty")]
    EmptyProjectLocation,
    #[error("project path cannot be empty or blank")]
    EmptyProjectPath,
    #[error("could not resolve current directory: {source}")]
    CurrentDirectory {
        #[source]
        source: std::io::Error,
    },
    #[error(
        "target path already exists as a file: {path}",
        path = display_project_path(path)
    )]
    TargetIsFile { path: PathBuf },
    #[error(
        "target directory must be empty: {path}",
        path = display_project_path(path)
    )]
    TargetNotEmpty { path: PathBuf },
    #[error(
        "project directory does not exist: {path}",
        path = display_project_path(path)
    )]
    ProjectMissing { path: PathBuf },
    #[error(
        "project path crosses a symbolic link or Windows reparse point: {path}",
        path = display_project_path(path)
    )]
    LinkedPath { path: PathBuf },
    #[error(
        "project manifest is missing: {path}",
        path = display_project_path(path)
    )]
    ManifestMissing { path: PathBuf },
    #[error(
        "project manifest exceeds the {max_bytes}-byte preflight limit: {path}",
        path = display_project_path(path)
    )]
    ManifestPreflightTooLarge { path: PathBuf, max_bytes: usize },
    #[error("project manifest failed: {source}")]
    Manifest {
        #[from]
        #[source]
        source: ProjectManifestError,
    },
    #[error("project generation preparation failed: {source}")]
    ProjectGeneration {
        #[from]
        #[source]
        source: AssetImportError,
    },
    #[error("scene asset target is invalid: {uri}; {reason}")]
    SceneTarget { uri: String, reason: &'static str },
    #[error(
        "scene asset already exists: {path}",
        path = display_project_path(path)
    )]
    SceneAlreadyExists { path: PathBuf },
    #[error("scene document operation failed: {source}")]
    SceneDocument {
        #[from]
        #[source]
        source: SceneProjectError,
    },
    #[error("scene catalog synchronization failed: {source}")]
    SceneCatalog {
        #[source]
        source: AssetImportError,
    },
    #[error("scene catalog {operation} failed: {source}")]
    SceneCatalogRuntime {
        operation: &'static str,
        #[source]
        source: CoreError,
    },
    #[error(
        "scene catalog synchronization failed: {catalog}; catalog reconciliation after source rollback also failed: {reconcile}"
    )]
    SceneCatalogReconcile {
        catalog: AssetImportError,
        #[source]
        reconcile: AssetImportError,
    },
    #[error(
        "scene catalog synchronization failed: {catalog}; scene source rollback also failed: {rollback}"
    )]
    SceneCatalogRollback {
        #[source]
        catalog: AssetImportError,
        rollback: Box<ProjectAuthorityError>,
    },
    #[error(
        "removing the published scene staging source failed: {cleanup}; scene source rollback also failed: {rollback}"
    )]
    SceneStagingCleanupRollback {
        #[source]
        cleanup: Box<ProjectAuthorityError>,
        rollback: Box<ProjectAuthorityError>,
    },
    #[error("project template pack failed: {source}")]
    TemplatePack {
        #[from]
        #[source]
        source: ProjectTemplatePackError,
    },
    #[error(
        "project filesystem operation {operation} failed for {path}: {source}",
        path = display_project_path(path)
    )]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "project commit failed for {target_path}, and restoring backup {backup_path} also failed: commit: {commit_source}; restore: {restore_source}",
        target_path = display_project_path(target),
        backup_path = display_project_path(backup)
    )]
    CommitRollbackFailed {
        target: PathBuf,
        backup: PathBuf,
        #[source]
        commit_source: std::io::Error,
        restore_source: std::io::Error,
    },
    #[error(
        "post-commit project rollback failed moving {from_path} to {to_path}; preserved empty-target backup: {backup_path:?}: {source}",
        from_path = display_project_path(from),
        to_path = display_project_path(to),
        backup_path = display_optional_project_path(backup)
    )]
    PostCommitRollbackFailed {
        from: PathBuf,
        to: PathBuf,
        backup: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },
}

fn display_project_path(path: &Path) -> String {
    ProjectPaths::display_path(path).display().to_string()
}

fn display_optional_project_path(path: &Option<PathBuf>) -> Option<String> {
    path.as_deref().map(display_project_path)
}

impl ProjectAuthorityError {
    pub(super) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ProjectAuthorityError;

    #[cfg(windows)]
    #[test]
    fn project_authority_error_displays_windows_operation_paths_without_verbatim_prefixes() {
        let error = ProjectAuthorityError::ManifestMissing {
            path: PathBuf::from(r"\\?\C:\ZirconBuilds\stage\project\zircon-project.toml"),
        };

        assert_eq!(
            error.to_string(),
            r"project manifest is missing: C:\ZirconBuilds\stage\project\zircon-project.toml"
        );
    }
}
