use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectValidation {
    Valid,
    MissingRoot,
    MissingManifest,
}

pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return ProjectValidation::MissingRoot;
    }
    if !path.join("zircon-project.toml").is_file() {
        return ProjectValidation::MissingManifest;
    }
    ProjectValidation::Valid
}
