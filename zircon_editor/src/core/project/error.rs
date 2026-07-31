use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime::asset::project::ProjectManifestError;
use zircon_runtime::asset::AssetImportError;
use zircon_runtime_interface::project::ProjectNameError;
use zircon_runtime_interface::project::ProjectTemplatePackError;

#[derive(Debug, Error)]
pub enum ProjectAuthorityError {
    #[error("project name is invalid: {source}")]
    ProjectName {
        #[from]
        #[source]
        source: ProjectNameError,
    },
    #[error("project location cannot be empty")]
    EmptyProjectLocation,
    #[error("project path cannot be empty or blank")]
    EmptyProjectPath,
    #[error("could not resolve current directory: {source}")]
    CurrentDirectory {
        #[source]
        source: std::io::Error,
    },
    #[error("target path already exists as a file: {path}")]
    TargetIsFile { path: PathBuf },
    #[error("target directory must be empty: {path}")]
    TargetNotEmpty { path: PathBuf },
    #[error("project directory does not exist: {path}")]
    ProjectMissing { path: PathBuf },
    #[error("project path crosses a symbolic link or Windows reparse point: {path}")]
    LinkedPath { path: PathBuf },
    #[error("project manifest is missing: {path}")]
    ManifestMissing { path: PathBuf },
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
    #[error("project template pack failed: {source}")]
    TemplatePack {
        #[from]
        #[source]
        source: ProjectTemplatePackError,
    },
    #[error("project filesystem operation {operation} failed for {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "project commit failed for {target}, and restoring backup {backup} also failed: commit: {commit_source}; restore: {restore_source}"
    )]
    CommitRollbackFailed {
        target: PathBuf,
        backup: PathBuf,
        #[source]
        commit_source: std::io::Error,
        restore_source: std::io::Error,
    },
    #[error(
        "post-commit project rollback failed moving {from} to {to}; preserved empty-target backup: {backup:?}: {source}"
    )]
    PostCommitRollbackFailed {
        from: PathBuf,
        to: PathBuf,
        backup: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },
    #[error("project session JSON decode failed: {source}")]
    SessionDecode {
        #[source]
        source: serde_json::Error,
    },
    #[error("project session JSON encode failed: {source}")]
    SessionEncode {
        #[source]
        source: serde_json::Error,
    },
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
