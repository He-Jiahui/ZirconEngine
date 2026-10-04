use std::path::Path;

use super::{editor_autosave_document_identity, next_autosave_sequence};

#[test]
fn foreground_save_job_is_bound_to_the_editor_manager_not_the_runtime_core() {
    let source = include_str!("../editor_document_autosave.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("autosave tests must remain separate from production code");
    let job = production
        .split_once("pub(super) struct ForegroundDocumentSaveJob {")
        .and_then(|(_, remainder)| remainder.split_once("impl ForegroundDocumentSaveJob"))
        .map(|(job, _)| job)
        .expect("foreground save job must remain a distinct owner");
    let run = production
        .split_once("impl EditorJob for ForegroundDocumentSaveJob {")
        .map(|(_, run)| run)
        .expect("foreground save job must retain its execution owner");

    assert!(job.contains("manager: Weak<EditorManager>"));
    assert!(run.contains("self.manager.upgrade()"));
    assert!(!job.contains("core:"));
    assert!(!production.contains("resolve_manager::<EditorManager>"));
}

#[test]
fn autosave_snapshot_sequences_are_strictly_monotonic_in_process() {
    let first = next_autosave_sequence();
    let second = next_autosave_sequence();

    assert!(second > first);
}

#[test]
fn autosave_document_identity_is_stable_for_a_project_relative_source() {
    let first = editor_autosave_document_identity(
        Path::new("project"),
        Path::new("assets/player.zui").to_path_buf(),
    )
    .unwrap();
    let second = editor_autosave_document_identity(
        Path::new("project"),
        Path::new("assets/player.zui").to_path_buf(),
    )
    .unwrap();

    assert_eq!(first.document, second.document);
    assert_eq!(first.source_path.as_path(), Path::new("assets/player.zui"));
}

#[test]
fn autosave_document_identity_resolves_relative_sources_from_the_project_root() {
    let project_root = std::env::current_dir()
        .unwrap()
        .join("autosave-relative-project");
    let relative = editor_autosave_document_identity(
        &project_root,
        Path::new("assets/player.zui").to_path_buf(),
    )
    .unwrap();
    let rooted =
        editor_autosave_document_identity(&project_root, project_root.join("assets/player.zui"))
            .unwrap();

    assert_eq!(relative.document, rooted.document);
    assert_eq!(
        relative.source_path.as_path(),
        Path::new("assets/player.zui")
    );
}
