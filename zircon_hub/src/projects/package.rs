use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::HubError;

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

pub fn package_project(request: &ProjectPackageRequest) -> Result<ProjectPackageReport, HubError> {
    if request.project_root.as_os_str().is_empty() || !request.project_root.is_dir() {
        return Err(HubError::message(
            "Project root is not available for packaging",
        ));
    }
    if request.output_root.as_os_str().is_empty() {
        return Err(HubError::message("Package output root is required"));
    }

    fs::create_dir_all(&request.output_root)?;
    reject_output_inside_project(&request.project_root, &request.output_root)?;

    let package_dir = unique_package_dir(request);
    let project_dir = package_dir.join(PACKAGE_PROJECT_DIR);
    fs::create_dir_all(&project_dir)?;
    let files_copied = copy_project_tree(&request.project_root, &project_dir)?;
    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    write_package_manifest(request, &manifest_path, files_copied)?;

    Ok(ProjectPackageReport {
        package_dir,
        manifest_path,
        files_copied,
    })
}

fn reject_output_inside_project(project_root: &Path, output_root: &Path) -> Result<(), HubError> {
    let project_root = project_root.canonicalize()?;
    let output_root = output_root.canonicalize()?;
    if output_root.starts_with(&project_root) {
        return Err(HubError::message(
            "Package output root must be outside the project directory",
        ));
    }
    Ok(())
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

fn copy_project_tree(source: &Path, destination: &Path) -> Result<usize, HubError> {
    let mut files_copied = 0;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            fs::create_dir_all(&target_path)?;
            files_copied += copy_project_tree(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
            files_copied += 1;
        }
    }
    Ok(files_copied)
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
mod tests {
    use super::*;

    #[test]
    fn package_project_copies_project_files_and_writes_manifest() {
        let root = temp_dir("package-source");
        let output = temp_dir("package-output");
        fs::write(root.join("zircon-project.toml"), "name = \"Demo\"").unwrap();
        fs::create_dir_all(root.join("Assets")).unwrap();
        fs::write(root.join("Assets").join("mesh.txt"), "mesh").unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("target").join("ignored.txt"), "ignored").unwrap();

        let request = ProjectPackageRequest {
            project_name: "Demo Project".to_string(),
            project_root: root.clone(),
            output_root: output.clone(),
            created_unix_ms: 42,
        };
        let report = package_project(&request).unwrap();

        assert!(report
            .package_dir
            .ends_with(Path::new("packages").join("demo-project-42")));
        assert!(report
            .package_dir
            .join(PACKAGE_PROJECT_DIR)
            .join("zircon-project.toml")
            .is_file());
        assert!(report
            .package_dir
            .join(PACKAGE_PROJECT_DIR)
            .join("Assets")
            .join("mesh.txt")
            .is_file());
        assert!(!report
            .package_dir
            .join(PACKAGE_PROJECT_DIR)
            .join("target")
            .exists());
        assert_eq!(report.files_copied, 2);
        assert!(fs::read_to_string(report.manifest_path)
            .unwrap()
            .contains("files_copied = 2"));

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(output).unwrap();
    }

    #[test]
    fn package_project_rejects_output_inside_project() {
        let root = temp_dir("package-source-inside");
        let output = root.join("build-output");
        let request = ProjectPackageRequest::new("Demo", root.clone(), output);

        let error = package_project(&request).unwrap_err();
        fs::remove_dir_all(root).unwrap();

        assert!(error.to_string().contains("outside the project directory"));
    }

    fn temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-{label}-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
