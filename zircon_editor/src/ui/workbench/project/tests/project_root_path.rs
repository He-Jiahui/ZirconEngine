use zircon_runtime::asset::project::{ProjectPaths, PROJECT_MANIFEST_FILE};
use zircon_runtime::scene::world::SceneProjectError;

use super::project_root_path;

#[cfg(windows)]
#[test]
fn project_root_path_rejects_drive_relative_paths() {
    let result = project_root_path(r"C:ambiguous-project-root");

    assert!(matches!(
        result,
        Err(SceneProjectError::Io(error)) if error.kind() == std::io::ErrorKind::InvalidInput
    ));
}

#[test]
fn project_root_path_keeps_a_manifest_named_directory_as_the_root() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    let location = std::env::temp_dir().join(format!(
        "zircon-workbench-project-root-{unique}-{}",
        std::process::id()
    ));
    let project_root = location.join(PROJECT_MANIFEST_FILE);
    std::fs::create_dir_all(&project_root).unwrap();

    assert_eq!(
        project_root_path(&project_root).unwrap(),
        ProjectPaths::resolve_existing_path(&project_root).unwrap()
    );
    std::fs::remove_dir_all(location).unwrap();
}
