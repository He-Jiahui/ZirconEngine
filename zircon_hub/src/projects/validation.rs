use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectValidation {
    Valid,
    MissingRoot,
    MissingManifest,
    InvalidManifest,
}

pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return ProjectValidation::MissingRoot;
    }
    if !path.join("zircon-project.toml").is_file() {
        return ProjectValidation::MissingManifest;
    }
    if !manifest_is_parsable(&path.join("zircon-project.toml")) {
        return ProjectValidation::InvalidManifest;
    }
    ProjectValidation::Valid
}

fn manifest_is_parsable(manifest_path: &Path) -> bool {
    fs::read_to_string(manifest_path)
        .ok()
        .is_some_and(|text| toml::from_str::<toml::Value>(&text).is_ok())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn validate_project_root_accepts_parsable_manifest() {
        let root = temp_dir("project-validation-valid");
        fs::write(root.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();

        assert_eq!(validate_project_root(&root), ProjectValidation::Valid);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_project_root_rejects_invalid_manifest_toml() {
        let root = temp_dir("project-validation-invalid-manifest");
        fs::write(root.join("zircon-project.toml"), "name = \"Game\n").unwrap();

        assert_eq!(
            validate_project_root(&root),
            ProjectValidation::InvalidManifest
        );

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-{label}-{}",
            crate::projects::now_unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
}
