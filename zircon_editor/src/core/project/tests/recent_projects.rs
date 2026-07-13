use zircon_runtime_interface::project::ProjectManifestSummary;

use super::super::{
    ProjectAuthority, RecentProjectEntry, RecentProjectValidation, StoredStartupSession,
};

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
fn recent_projection_uses_dynamic_validation_without_persisting_it() {
    let stored = StoredStartupSession {
        last_project_path: None,
        recent_projects: vec![RecentProjectEntry {
            summary: summary("Game"),
            path: "E:/Projects/Game".to_string(),
            last_opened_unix_ms: 10,
            validation: RecentProjectValidation::Valid,
        }
        .into_stored()],
    };

    let recent = ProjectAuthority::default()
        .recent_projects_with_validation(&stored, |_| RecentProjectValidation::Missing);

    assert_eq!(recent[0].summary.name, "Game");
    assert_eq!(recent[0].validation, RecentProjectValidation::Missing);
}

fn summary(name: &str) -> ProjectManifestSummary {
    ProjectManifestSummary {
        name: name.to_string(),
        engine_version_req: Some(">=0.1.0".to_string()),
        default_scene: "res://scenes/main.scene.toml".to_string(),
        format_version: 2,
    }
}
