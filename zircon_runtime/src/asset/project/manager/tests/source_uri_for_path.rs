use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};

use super::ProjectManager;

static NEXT_TEST_PROJECT: AtomicU64 = AtomicU64::new(1);

#[cfg(any(unix, windows))]
#[test]
fn physical_source_under_an_alias_root_keeps_its_project_uri_identity() {
    let parent = unique_temp_project_root("source-uri-alias");
    let physical_root = parent.join("physical-project");
    write_renderable_empty_template(&physical_root);
    let alias_root = parent.join("project-alias");
    create_directory_alias(&physical_root, &alias_root);

    let manager = ProjectManager::open(&alias_root).unwrap();
    let uri = manager
        .project_uri_for_source_path(&physical_root.join("assets/models/cube.obj"))
        .unwrap();

    assert_eq!(uri.to_string(), "res://models/cube.obj");
    drop(manager);
    remove_directory_alias(&alias_root);
    fs::remove_dir_all(parent).unwrap();
}

fn write_renderable_empty_template(root: &Path) {
    let rendered =
        render_project_template(ProjectTemplateId::RenderableEmpty, "SourceUriAlias").unwrap();
    for entry in rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(destination, entry.bytes).unwrap();
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let sequence = NEXT_TEST_PROJECT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_project_manager_{label}_{timestamp}_{sequence}"
    ))
}

#[cfg(unix)]
fn create_directory_alias(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(unix)]
fn remove_directory_alias(link: &Path) {
    fs::remove_file(link).unwrap();
}

#[cfg(windows)]
fn create_directory_alias(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "create project alias fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
fn remove_directory_alias(link: &Path) {
    fs::remove_dir(link).unwrap();
}
