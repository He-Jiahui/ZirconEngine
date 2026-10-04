use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::HubError;
use crate::state::{
    DeliveryMessageId, HubMessage, HubMessageId, TaskCancellationToken, TaskExecutionOutcome,
};

use super::local_paths::{
    cleanup_dir_on_error, create_owned_dir, reject_inside_root, remove_owned_dir,
};
use super::now_unix_ms;

const PACKAGE_ROOT_DIR: &str = "packages";
const PACKAGE_PROJECT_DIR: &str = "project";
const PACKAGE_MANIFEST_FILE: &str = "zircon-package.toml";
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPackageRequest {
    pub project_name: String,
    pub project_root: PathBuf,
    pub output_root: PathBuf,
    pub created_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPackageReport {
    pub package_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub files_copied: usize,
}

impl ProjectPackageReport {
    pub(crate) fn remove_owned_output(&self) -> Result<(), HubError> {
        remove_owned_dir(&self.package_dir)
    }
}

#[derive(Serialize)]
struct ProjectPackageManifest {
    package_name: String,
    source_project: String,
    created_unix_ms: u64,
    project_dir: String,
    files_copied: usize,
}

impl ProjectPackageRequest {
    pub fn new(
        project_name: impl Into<String>,
        project_root: impl Into<PathBuf>,
        output_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            project_name: project_name.into(),
            project_root: project_root.into(),
            output_root: output_root.into(),
            created_unix_ms: now_unix_ms(),
        }
    }
}

pub fn package_project(
    request: &ProjectPackageRequest,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<ProjectPackageReport>, HubError> {
    if cancellation.is_cancellation_requested() {
        return Ok(TaskExecutionOutcome::Cancelled);
    }
    if request.project_root.as_os_str().is_empty() || !request.project_root.is_dir() {
        return Err(HubError::status(
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::ProjectRootUnavailable,
            )),
            None,
        ));
    }
    if request.output_root.as_os_str().is_empty() {
        return Err(HubError::status(
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::PackageOutputRootRequired,
            )),
            None,
        ));
    }

    reject_output_inside_project(&request.project_root, &request.output_root)?;
    fs::create_dir_all(&request.output_root)?;

    let package_dir = unique_package_dir(request);
    fs::create_dir_all(
        package_dir
            .parent()
            .expect("package directory should have a parent"),
    )?;
    create_owned_dir(&package_dir, || {
        HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::PackageDirectoryAlreadyExists),
            [package_dir.to_string_lossy().into_owned()],
        )
    })?;

    match fill_package_dir(request, &package_dir, cancellation) {
        Ok(TaskExecutionOutcome::Completed(report)) => Ok(TaskExecutionOutcome::Completed(report)),
        Ok(TaskExecutionOutcome::Cancelled) => {
            remove_owned_dir(&package_dir)?;
            Ok(TaskExecutionOutcome::Cancelled)
        }
        Err(error) => cleanup_dir_on_error(&package_dir, Err(error)),
    }
}

fn fill_package_dir(
    request: &ProjectPackageRequest,
    package_dir: &Path,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<ProjectPackageReport>, HubError> {
    if cancellation.is_cancellation_requested() {
        return Ok(TaskExecutionOutcome::Cancelled);
    }
    let project_dir = package_dir.join(PACKAGE_PROJECT_DIR);
    fs::create_dir(&project_dir)?;
    let files_copied = match copy_project_tree(&request.project_root, &project_dir, cancellation)? {
        TaskExecutionOutcome::Completed(files_copied) => files_copied,
        TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
    };
    if cancellation.is_cancellation_requested() {
        return Ok(TaskExecutionOutcome::Cancelled);
    }
    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    write_package_manifest(request, &manifest_path, files_copied)?;

    Ok(TaskExecutionOutcome::Completed(ProjectPackageReport {
        package_dir: package_dir.to_path_buf(),
        manifest_path,
        files_copied,
    }))
}

fn reject_output_inside_project(project_root: &Path, output_root: &Path) -> Result<(), HubError> {
    reject_inside_root(
        project_root,
        output_root,
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::PackageOutputOutsideProject,
        )),
    )
}

fn unique_package_dir(request: &ProjectPackageRequest) -> PathBuf {
    let base_name = package_basename(&request.project_name);
    request
        .output_root
        .join(PACKAGE_ROOT_DIR)
        .join(format!("{base_name}-{}", request.created_unix_ms))
}

fn package_basename(project_name: &str) -> String {
    let sanitized: String = project_name
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "zircon-project".to_string()
    } else {
        sanitized.to_ascii_lowercase()
    }
}

fn copy_project_tree(
    source: &Path,
    destination: &Path,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<usize>, HubError> {
    let mut files_copied = 0;
    for entry in fs::read_dir(source)? {
        if cancellation.is_cancellation_requested() {
            return Ok(TaskExecutionOutcome::Cancelled);
        }
        let entry = entry?;
        let source_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            fs::create_dir_all(&target_path)?;
            match copy_project_tree(&source_path, &target_path, cancellation)? {
                TaskExecutionOutcome::Completed(copied) => files_copied += copied,
                TaskExecutionOutcome::Cancelled => return Ok(TaskExecutionOutcome::Cancelled),
            }
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
            files_copied += 1;
        }
    }
    Ok(TaskExecutionOutcome::Completed(files_copied))
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn write_package_manifest(
    request: &ProjectPackageRequest,
    manifest_path: &Path,
    files_copied: usize,
) -> Result<(), HubError> {
    let manifest = ProjectPackageManifest {
        package_name: request.project_name.clone(),
        source_project: request.project_root.to_string_lossy().into_owned(),
        created_unix_ms: request.created_unix_ms,
        project_dir: PACKAGE_PROJECT_DIR.to_string(),
        files_copied,
    };
    fs::write(manifest_path, toml::to_string_pretty(&manifest)?)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/package.rs"]
mod tests;
