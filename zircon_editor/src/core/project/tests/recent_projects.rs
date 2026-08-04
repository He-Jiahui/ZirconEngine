use std::fs;
use std::path::PathBuf;

use zircon_runtime_interface::project::ProjectManifestSummary;

use super::super::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, RecentProjectEntry,
    RecentProjectValidation, StoredRecentProjectEntry, StoredStartupSession,
};
use super::temp_root;

#[test]
fn stored_recent_project_roundtrip_keeps_manifest_summary_as_identity() {
    let mut stored = StoredStartupSession::default();
    ProjectAuthority::default().remember_recent_project(
        &mut stored,
        "E:/Projects/Game",
        summary("Manifest Name"),
        42,
    );

    let encoded = serde_json::to_value(&stored).unwrap();
    let decoded: StoredStartupSession = serde_json::from_value(encoded).unwrap();

    assert_eq!(decoded.recent_projects[0].summary.name, "Manifest Name");
    assert_eq!(
        decoded.last_project_path.as_deref(),
        Some("E:/Projects/Game")
    );
}

#[test]
fn remembering_same_path_refreshes_summary_and_keeps_newest_record() {
    let authority = ProjectAuthority::default();
    let mut stored = StoredStartupSession::default();
    authority.remember_recent_project(&mut stored, "E:/Projects/Game", summary("Old"), 1);
    authority.remember_recent_project(&mut stored, "E:/Projects/Game", summary("Current"), 2);

    assert_eq!(stored.recent_projects.len(), 1);
    assert_eq!(stored.recent_projects[0].summary.name, "Current");
    assert_eq!(stored.recent_projects[0].last_opened_unix_ms, 2);
}

#[test]
fn remembering_a_canonical_root_replaces_a_legacy_manifest_recent_entry() {
    let (location, root_path, project_summary) = create_recent_test_project(
        "remember-legacy-manifest-recent-entry",
        "Remember Legacy Manifest Project",
    );
    let authority = ProjectAuthority::default();
    let root = root_path.to_string_lossy().into_owned();
    let manifest = root_path.join("zircon-project.toml");
    let manifest_path = manifest.to_string_lossy().into_owned();
    let mut stored = legacy_manifest_recent_entry(&manifest_path, project_summary.clone(), 1);

    authority.remember_recent_project(&mut stored, &root, project_summary, 2);

    assert_eq!(stored.recent_projects.len(), 1);
    assert_eq!(stored.recent_projects[0].path, root);
    assert_eq!(stored.recent_projects[0].last_opened_unix_ms, 2);
    assert_eq!(stored.last_project_path.as_deref(), Some(root.as_str()));
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn remembering_a_manifest_path_persists_the_canonical_project_root() {
    let (location, root_path, project_summary) = create_recent_test_project(
        "remember-manifest-path-as-canonical-root",
        "Manifest Root Project",
    );
    let authority = ProjectAuthority::default();
    let root = root_path.to_string_lossy().into_owned();
    let manifest = root_path.join("zircon-project.toml");
    let mut stored = StoredStartupSession::default();

    authority.remember_recent_project(
        &mut stored,
        manifest.to_string_lossy().as_ref(),
        project_summary,
        42,
    );

    assert_eq!(stored.recent_projects.len(), 1);
    assert_eq!(stored.recent_projects[0].path, root);
    assert_eq!(stored.last_project_path.as_deref(), Some(root.as_str()));
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn recent_projection_uses_dynamic_validation_without_persisting_it() {
    let stored = StoredStartupSession {
        last_project_path: None,
        recent_projects: vec![
            RecentProjectEntry {
                summary: summary("Game"),
                path: "E:/Projects/Game".to_string(),
                last_opened_unix_ms: 10,
                validation: RecentProjectValidation::Valid,
            }
            .into_stored(),
        ],
    };

    let recent = ProjectAuthority::default()
        .recent_projects_with_validation(&stored, |_| RecentProjectValidation::Missing);

    assert_eq!(recent[0].summary.name, "Game");
    assert_eq!(recent[0].validation, RecentProjectValidation::Missing);
}

#[test]
fn forgetting_a_manifest_path_removes_the_canonical_recent_project_root() {
    let (location, root_path, project_summary) = create_recent_test_project(
        "forget-recent-project-manifest-path",
        "Forget Manifest Project",
    );
    let authority = ProjectAuthority::default();
    let root = root_path.to_string_lossy().into_owned();
    let manifest = root_path.join("zircon-project.toml");
    let mut stored = StoredStartupSession::default();
    authority.remember_recent_project(&mut stored, &root, project_summary, 42);

    authority.forget_recent_project(&mut stored, manifest.to_string_lossy().as_ref());

    assert!(stored.recent_projects.is_empty());
    assert_eq!(stored.last_project_path, None);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn forgetting_a_canonical_root_removes_a_legacy_manifest_recent_entry() {
    let (location, root_path, project_summary) = create_recent_test_project(
        "forget-legacy-manifest-recent-entry",
        "Forget Legacy Manifest Project",
    );
    let authority = ProjectAuthority::default();
    let root = root_path.to_string_lossy().into_owned();
    let manifest = root_path.join("zircon-project.toml");
    let mut stored =
        legacy_manifest_recent_entry(manifest.to_string_lossy().as_ref(), project_summary, 42);

    authority.forget_recent_project(&mut stored, &root);

    assert!(stored.recent_projects.is_empty());
    assert_eq!(stored.last_project_path, None);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn startup_session_migrates_legacy_recent_entry_from_its_project_manifest() {
    let (location, root_path, project_summary) =
        create_recent_test_project("legacy-recent-session", "Legacy Session Project");
    let authority = ProjectAuthority::default();
    let path = root_path.to_string_lossy().into_owned();
    let legacy = serde_json::json!({
        "last_project_path": path,
        "recent_projects": [{
            "path": path,
            "last_opened_unix_ms": 42
        }]
    });

    let session = authority.decode_startup_session(legacy).unwrap();

    assert_eq!(session.last_project_path.as_deref(), Some(path.as_str()));
    assert_eq!(session.recent_projects.len(), 1);
    assert_eq!(session.recent_projects[0].summary, project_summary);
    assert_eq!(session.recent_projects[0].last_opened_unix_ms, 42);
    fs::remove_dir_all(location).unwrap();
}

fn legacy_manifest_recent_entry(
    manifest_path: &str,
    summary: ProjectManifestSummary,
    last_opened_unix_ms: u64,
) -> StoredStartupSession {
    StoredStartupSession {
        last_project_path: Some(manifest_path.to_string()),
        recent_projects: vec![StoredRecentProjectEntry {
            summary,
            path: manifest_path.to_string(),
            last_opened_unix_ms,
        }],
    }
}

fn create_recent_test_project(
    case_name: &str,
    project_name: &str,
) -> (PathBuf, PathBuf, ProjectManifestSummary) {
    let location = temp_root(case_name);
    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: project_name.to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    (location, created.root, created.summary)
}

fn summary(name: &str) -> ProjectManifestSummary {
    ProjectManifestSummary {
        name: name.to_string(),
        engine_version_req: Some(">=0.1.0".to_string()),
        default_scene: "res://scenes/main.scene.toml".to_string(),
        format_version: 2,
    }
}
