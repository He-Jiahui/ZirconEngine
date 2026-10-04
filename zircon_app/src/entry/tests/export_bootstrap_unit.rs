use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use super::discover_export_root_from_paths;

#[cfg(any(unix, windows))]
#[test]
fn export_root_discovery_keeps_the_physical_identity_of_a_product_alias() {
    let parent = unique_export_root("alias");
    let physical_root = parent.join("physical-export");
    fs::create_dir_all(physical_root.join("plugins")).unwrap();
    fs::create_dir_all(physical_root.join("bin")).unwrap();
    fs::write(physical_root.join("plugins/native_plugins.toml"), []).unwrap();
    let alias = parent.join("export-alias");
    create_directory_link(&physical_root, &alias);
    let working_directory = parent.join("working-directory");
    fs::create_dir_all(&working_directory).unwrap();

    let actual =
        discover_export_root_from_paths(&alias.join("bin/exported-product"), &working_directory)
            .expect("export root discovery should resolve the product alias");
    let expected = ProjectPaths::resolve_existing_path(&physical_root).unwrap();

    fs::remove_dir_all(&parent).unwrap();
    assert_eq!(actual, expected);
}

fn unique_export_root(case_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "zircon-export-bootstrap-{case_name}-{}-{timestamp}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create export-root alias fixture");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for export-root alias fixture");
    assert!(
        output.status.success(),
        "create export-root junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
