mod boundary;
mod directory_transaction;
mod preflight;
mod root_resolution;
mod scene_document;
mod template_creation;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::ProjectPaths;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

#[test]
fn project_authority_fixture_roots_follow_the_resolved_test_binary_directory() {
    let path = temp_root("physical-root");
    let executable = std::env::current_exe().expect("locate the project-authority test executable");
    let binary_directory = executable
        .parent()
        .expect("project-authority test executable must have a parent directory");
    let resolved_binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve project-authority test binary directory");

    assert!(
        path.starts_with(resolved_binary_directory.operation_path()),
        "project-authority fixture output must retain the test binary's physical output root"
    );
    std::fs::remove_dir_all(path).expect("remove project-authority fixture root");
}

fn temp_root(label: &str) -> PathBuf {
    let executable = std::env::current_exe().expect("locate the project-authority test executable");
    let binary_directory = executable
        .parent()
        .expect("project-authority test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve project-authority test binary directory");
    let path = binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(format!(
            "zircon-editor-project-authority-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}
