use std::{fs, path::PathBuf};

use super::*;

#[test]
fn validate_project_root_accepts_parsable_manifest() {
    let root = temp_dir("project-validation-valid");
    install_shared_fixture(&root, "v1");

    assert_eq!(validate_project_root(&root), ProjectValidation::Valid);

    install_shared_fixture(&root, "v2");
    assert_eq!(validate_project_root(&root), ProjectValidation::Valid);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn validate_project_root_rejects_future_or_bad_shape_manifests() {
    let root = temp_dir("project-validation-future-manifest");
    install_shared_fixture(&root, "future");
    assert_eq!(
        validate_project_root(&root),
        ProjectValidation::InvalidManifest
    );

    install_shared_fixture(&root, "invalid");
    assert_eq!(
        validate_project_root(&root),
        ProjectValidation::InvalidManifest
    );

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

fn install_shared_fixture(root: &Path, version: &str) {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("serialization")
        .join("project-manifest")
        .join(version)
        .join("zircon-project.toml");
    fs::copy(fixture, root.join("zircon-project.toml")).unwrap();
}
