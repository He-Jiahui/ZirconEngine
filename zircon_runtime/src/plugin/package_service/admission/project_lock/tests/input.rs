use super::*;
use zircon_runtime_interface::project::{ProjectGuid, PROJECT_MANIFEST_FORMAT_VERSION};

#[test]
fn project_manifest_bytes_and_guid_are_both_required_for_admission() {
    let target =
        std::path::PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("managed target"));
    let root = target.join(format!("package-lock-input-{}", ProjectGuid::new()));
    std::fs::create_dir_all(&root).unwrap();
    let guid = ProjectGuid::new();
    let bytes = format!(
        "name = 'Local'\ndefault_scene = 'res://scenes/main.scene.toml'\nformat_version = {}\nlibrary_version = 1\nproject_guid = '{guid}'\nasset_roots = ['res']\n",
        PROJECT_MANIFEST_FORMAT_VERSION,
    ).into_bytes();
    std::fs::write(root.join("zircon-project.toml"), &bytes).unwrap();
    let mut expected = ProjectPackageLockProject {
        project_guid: guid,
        manifest_digest: ProjectManifestDigest::from_bytes(&bytes),
    };
    let input = read_project_manifest(&root, &expected).unwrap();
    assert!(input.manifest.plugins.selections.is_empty());
    drop(input);
    expected.project_guid = ProjectGuid::new();
    assert!(matches!(
        read_project_manifest(&root, &expected),
        Err(ProjectPackageLockProviderError::InvalidProjectIdentity)
    ));
    expected.project_guid = guid;
    std::fs::write(
        root.join("zircon-project.toml"),
        [bytes, b"\n".to_vec()].concat(),
    )
    .unwrap();
    assert!(matches!(
        read_project_manifest(&root, &expected),
        Err(ProjectPackageLockProviderError::InvalidProjectIdentity)
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_project_manifest_cannot_be_replaced_by_caller_selections() {
    let root = std::env::temp_dir().join(format!("missing-lock-project-{}", ProjectGuid::new()));
    let expected = ProjectPackageLockProject {
        project_guid: ProjectGuid::new(),
        manifest_digest: ProjectManifestDigest::from_bytes(b"not a project"),
    };
    assert!(matches!(
        read_project_manifest(&root, &expected),
        Err(ProjectPackageLockProviderError::InvalidProjectIdentity)
    ));
}
