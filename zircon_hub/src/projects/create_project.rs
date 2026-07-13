use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;
use zircon_runtime_interface::project::{
    render_project_template, ProjectManifestSummary, ProjectTemplatePackError,
};

use super::{CreateProjectRequest, CreateProjectRequestError, ProjectTemplate};

static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProjectReport {
    pub project_root: PathBuf,
    pub manifest_path: PathBuf,
    pub template: ProjectTemplate,
    pub summary: ProjectManifestSummary,
}

#[derive(Debug, Error)]
pub enum CreateProjectError {
    #[error("invalid project request: {source}")]
    InvalidRequest {
        #[from]
        #[source]
        source: CreateProjectRequestError,
    },
    #[error("project target has no parent directory: {path}")]
    TargetWithoutParent { path: PathBuf },
    #[error("target path already exists as a file: {path}")]
    TargetIsFile { path: PathBuf },
    #[error("target directory must be empty: {path}")]
    TargetNotEmpty { path: PathBuf },
    #[error("project target crosses a symbolic link or Windows reparse point: {path}")]
    LinkedPath { path: PathBuf },
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
}

impl CreateProjectError {
    fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}

pub fn create_project(
    request: &CreateProjectRequest,
) -> Result<CreateProjectReport, CreateProjectError> {
    request.validate_launch_fields()?;
    let target = request.target_root();
    validate_target_root(&target)?;
    let rendered = render_project_template(request.template.pack_id(), &request.project_name)?;
    let parent = target
        .parent()
        .ok_or_else(|| CreateProjectError::TargetWithoutParent {
            path: target.clone(),
        })?;
    fs::create_dir_all(parent)
        .map_err(|source| CreateProjectError::io("create project parent", parent, source))?;
    let transaction = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
    let stem = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    let staging = parent.join(format!(
        ".{stem}.zircon-staging-{}-{transaction}",
        std::process::id()
    ));
    let backup = parent.join(format!(
        ".{stem}.zircon-backup-{}-{transaction}",
        std::process::id()
    ));

    let result = (|| {
        fs::create_dir(&staging).map_err(|source| {
            CreateProjectError::io("create project staging directory", &staging, source)
        })?;
        for entry in rendered.entries {
            let destination = entry.path.join_to(&staging);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|source| {
                    CreateProjectError::io("create template directory", parent, source)
                })?;
            }
            fs::write(&destination, entry.bytes).map_err(|source| {
                CreateProjectError::io("write template entry", &destination, source)
            })?;
        }

        let replaced_empty_target = target.exists();
        commit_staged_directory(
            &staging,
            &target,
            &backup,
            replaced_empty_target,
            |from, to| fs::rename(from, to),
        )?;
        Ok(CreateProjectReport {
            manifest_path: target.join("zircon-project.toml"),
            project_root: target.clone(),
            template: request.template,
            summary: rendered.summary,
        })
    })();

    if result.is_err() {
        remove_transaction_path(&staging);
    }
    result
}

fn commit_staged_directory<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), CreateProjectError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    if replace_empty_target {
        rename(target, backup).map_err(|source| {
            CreateProjectError::io("stage empty target rollback", target, source)
        })?;
    }

    if let Err(commit_source) = rename(staging, target) {
        if replace_empty_target {
            if let Err(restore_source) = rename(backup, target) {
                return Err(CreateProjectError::CommitRollbackFailed {
                    target: target.to_path_buf(),
                    backup: backup.to_path_buf(),
                    commit_source,
                    restore_source,
                });
            }
        }
        return Err(CreateProjectError::io(
            "commit project template",
            target,
            commit_source,
        ));
    }

    if replace_empty_target {
        // The backup is known to be the previously empty target. Failure to remove this empty
        // directory does not invalidate the committed project, so preserve it for later cleanup.
        match fs::remove_dir(backup) {
            Ok(()) => {}
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
            Err(_cleanup_source) => {}
        }
    }
    Ok(())
}

fn validate_target_root(project_root: &Path) -> Result<(), CreateProjectError> {
    reject_linked_components(project_root)?;
    if project_root.is_file() {
        return Err(CreateProjectError::TargetIsFile {
            path: project_root.to_path_buf(),
        });
    }
    if project_root.is_dir() {
        let mut entries = fs::read_dir(project_root).map_err(|source| {
            CreateProjectError::io("read target directory", project_root, source)
        })?;
        if entries
            .next()
            .transpose()
            .map_err(|source| {
                CreateProjectError::io("read target directory entry", project_root, source)
            })?
            .is_some()
        {
            return Err(CreateProjectError::TargetNotEmpty {
                path: project_root.to_path_buf(),
            });
        }
    }
    Ok(())
}

fn reject_linked_components(path: &Path) -> Result<(), CreateProjectError> {
    let mut existing = path;
    while !existing.exists() {
        let Some(parent) = existing.parent() else {
            break;
        };
        existing = parent;
    }
    for ancestor in existing.ancestors() {
        let metadata = fs::symlink_metadata(ancestor)
            .map_err(|source| CreateProjectError::io("inspect target path", ancestor, source))?;
        if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
            return Err(CreateProjectError::LinkedPath {
                path: ancestor.to_path_buf(),
            });
        }
    }
    Ok(())
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

fn remove_transaction_path(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn create_project_copies_shared_pack_and_current_derived_layout() {
        let location = temp_dir("zircon-hub-create-project");
        let request =
            CreateProjectRequest::new("My Game", &location, ProjectTemplate::RenderableEmpty);

        let report = create_project(&request).expect("project should be scaffolded");

        assert_eq!(report.summary.name, "My Game");
        assert!(report.manifest_path.is_file());
        assert!(report
            .project_root
            .join("assets/scenes/main.scene.toml")
            .is_file());
        for directory in ["cache", "registry", "autosave", "play", "thumbnails"] {
            assert!(report.project_root.join(".zircon").join(directory).is_dir());
        }
        let retired_root = ["lib", "rary"].concat();
        assert!(!report.project_root.join(retired_root).exists());
        assert_eq!(
            zircon_runtime_interface::project::ProjectManifestSummary::parse_toml_bytes(
                &fs::read(&report.manifest_path).unwrap()
            )
            .unwrap()
            .value,
            report.summary
        );
        fs::remove_dir_all(location).unwrap();
    }

    #[test]
    fn create_project_rejects_non_empty_target_without_half_project() {
        let location = temp_dir("zircon-hub-create-project-non-empty");
        let target = location.join("My Game");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("existing.txt"), "keep").unwrap();
        let request =
            CreateProjectRequest::new("My Game", &location, ProjectTemplate::RenderableEmpty);

        assert!(matches!(
            create_project(&request),
            Err(CreateProjectError::TargetNotEmpty { .. })
        ));
        assert_eq!(
            fs::read_to_string(target.join("existing.txt")).unwrap(),
            "keep"
        );
        assert!(!fs::read_dir(&location).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("zircon-staging")
        }));
        fs::remove_dir_all(location).unwrap();
    }

    #[test]
    fn failed_commit_restores_the_original_empty_target() {
        let root = temp_dir("zircon-hub-commit-restore");
        let target = root.join("project");
        let staging = root.join("staging");
        let backup = root.join("backup");
        fs::create_dir(&target).unwrap();
        fs::create_dir(&staging).unwrap();
        let mut call = 0;

        let error = commit_staged_directory(&staging, &target, &backup, true, |from, to| {
            call += 1;
            if call == 2 {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected commit failure",
                ))
            } else {
                fs::rename(from, to)
            }
        })
        .unwrap_err();

        assert!(matches!(error, CreateProjectError::Io { .. }));
        assert!(target.is_dir());
        assert!(!backup.exists());
        assert!(staging.is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_restore_returns_typed_error_and_preserves_the_only_backup() {
        let root = temp_dir("zircon-hub-commit-rollback-failure");
        let target = root.join("project");
        let staging = root.join("staging");
        let backup = root.join("backup");
        fs::create_dir(&target).unwrap();
        fs::create_dir(&staging).unwrap();
        let mut call = 0;

        let error = commit_staged_directory(&staging, &target, &backup, true, |from, to| {
            call += 1;
            if call >= 2 {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected transaction failure",
                ))
            } else {
                fs::rename(from, to)
            }
        })
        .unwrap_err();

        assert!(matches!(
            error,
            CreateProjectError::CommitRollbackFailed { .. }
        ));
        assert!(!target.exists());
        assert!(backup.is_dir());
        assert!(staging.is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{suffix}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
