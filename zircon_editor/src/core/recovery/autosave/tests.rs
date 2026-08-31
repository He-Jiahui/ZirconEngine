use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectPaths;

use super::{
    AutosaveDocumentId, AutosaveExtension, AutosaveSnapshotProvenance, AutosaveSourceDigest,
    AutosaveStore,
};
use crate::core::recovery::AutosaveSourcePath;

#[test]
fn persisted_sequence_advances_a_lower_post_restart_proposal() {
    let project = unique_autosave_root("sequence-restart");
    let store = AutosaveStore::new(&project);
    let source = AutosaveSourcePath::parse("assets/player.zui").unwrap();
    let document = AutosaveDocumentId::from_source_path(&source);
    let extension = AutosaveExtension::parse("zui").unwrap();
    let provenance = snapshot_provenance();
    let first_sequence = store.next_sequence(&document, 100).unwrap();
    let first = store
        .write_snapshot(
            &document,
            &source,
            first_sequence,
            &extension,
            &provenance,
            b"first",
        )
        .unwrap();
    let second_sequence = store.next_sequence(&document, 1).unwrap();
    let second = store
        .write_snapshot(
            &document,
            &source,
            second_sequence,
            &extension,
            &provenance,
            b"second",
        )
        .unwrap();
    assert!(first.ends_with("100.zui"));
    assert!(second.ends_with("101.zui"));
    fs::remove_dir_all(project).unwrap();
}

#[test]
fn document_id_is_stable_for_a_normalized_project_relative_source() {
    let first = AutosaveSourcePath::parse("assets/ui/panel.zui").unwrap();
    let same = AutosaveSourcePath::parse("assets/ui/panel.zui").unwrap();
    let other = AutosaveSourcePath::parse("assets/ui/other.zui").unwrap();

    assert_eq!(
        AutosaveDocumentId::from_source_path(&first),
        AutosaveDocumentId::from_source_path(&same)
    );
    assert_ne!(
        AutosaveDocumentId::from_source_path(&first),
        AutosaveDocumentId::from_source_path(&other)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn autosave_root_keeps_the_physical_project_identity() {
    let parent = unique_autosave_root("physical-identity");
    let physical_project = parent.join("physical-project");
    fs::create_dir_all(&physical_project).unwrap();
    let project_alias = parent.join("project-alias");
    create_directory_link(&physical_project, &project_alias);

    let autosave_root = AutosaveStore::new(&project_alias).autosave_root();
    let expected = ProjectPaths::resolve_existing_path(&physical_project)
        .unwrap()
        .join(".zircon/autosave");

    fs::remove_dir_all(&parent).unwrap();
    assert_eq!(autosave_root, expected);
}

fn unique_autosave_root(case_name: &str) -> PathBuf {
    std::env::current_dir()
        .expect("current directory should be available")
        .join("target")
        .join(format!(
            "zircon-editor-autosave-store-{case_name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
}

fn snapshot_provenance() -> AutosaveSnapshotProvenance {
    AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing())
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create autosave project alias");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for autosave project alias");
    assert!(
        output.status.success(),
        "create autosave project junction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
