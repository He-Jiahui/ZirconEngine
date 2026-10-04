use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use super::{runtime_executable_next_to_path, ProcessPlayBackendInstallError};

#[test]
fn runtime_executable_resolution_rejects_a_path_without_an_install_parent() {
    let error = runtime_executable_next_to_path(Path::new(""))
        .expect_err("an empty executable path cannot identify an installation directory");

    assert!(matches!(
        error,
        ProcessPlayBackendInstallError::MissingEditorInstallDirectory
    ));
}

#[cfg(any(unix, windows))]
#[test]
fn play_runtime_executable_uses_the_physical_editor_install_identity() {
    let parent = unique_play_install_root("alias");
    let physical_install = parent.join("physical-install");
    fs::create_dir_all(&physical_install).unwrap();
    let install_alias = parent.join("install-alias");
    create_directory_link(&physical_install, &install_alias);

    let actual = runtime_executable_next_to_path(
        &install_alias.join(format!("zircon_editor{}", std::env::consts::EXE_SUFFIX)),
    )
    .expect("runtime executable should resolve from the editor installation");
    let expected = ProjectPaths::resolve_existing_path(&physical_install)
        .unwrap()
        .join(format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX));

    fs::remove_dir_all(&parent).unwrap();
    assert_eq!(actual, expected);
}

fn unique_play_install_root(case_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-play-install-{case_name}-{}-{timestamp}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create editor-install alias fixture");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for editor-install alias fixture");
    assert!(
        output.status.success(),
        "create editor-install junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
