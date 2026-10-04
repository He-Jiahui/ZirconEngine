use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use super::SettingsPaths;

#[cfg(any(unix, windows))]
#[test]
fn project_settings_path_keeps_the_physical_project_identity() {
    let parent = unique_settings_project_root("physical-identity");
    let physical_project = parent.join("physical-project");
    fs::create_dir_all(&physical_project).unwrap();
    let project_alias = parent.join("project-alias");
    create_directory_link(&physical_project, &project_alias);

    let paths = SettingsPaths::from_roots(parent.join("user"), Some(&project_alias));
    let expected = ProjectPaths::resolve_existing_path(&physical_project)
        .unwrap()
        .join(".zircon/settings.toml");

    fs::remove_dir_all(&parent).unwrap();
    assert_eq!(paths.project(), Some(expected.as_path()));
}

fn unique_settings_project_root(case_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-settings-{case_name}-{}-{timestamp}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create settings project alias");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for settings project alias");
    assert!(
        output.status.success(),
        "create settings project junction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
