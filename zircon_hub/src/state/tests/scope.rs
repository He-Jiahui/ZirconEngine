use super::*;
use crate::projects::{metadata_for_path_mut, ProjectMetadataMap, RecentProject};

fn engine(id: &str) -> SourceEngineInstall {
    SourceEngineInstall {
        id: id.to_string(),
        display_name: format!("{id} Engine"),
        source_dir: PathBuf::from(format!("E:/{id}")),
        output_dir: PathBuf::from(format!("E:/out/{id}")),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    }
}

#[test]
fn scope_prefers_selected_project_and_project_bound_engine() {
    let projects = vec![
        RecentProject::fixture("Latest", "E:/Projects/Latest", 20),
        RecentProject::fixture("Selected", "E:/Projects/Selected", 10),
    ];
    let engines = vec![engine("local")];
    let mut metadata = ProjectMetadataMap::new();
    metadata_for_path_mut(&mut metadata, "E:/Projects/Selected").engine_id =
        Some("local".to_string());

    let scope = HubScope::resolve(
        Some(Path::new("E:/Projects/Selected")),
        &projects,
        &metadata,
        &engines,
        None,
    );

    assert_eq!(scope.selected_project().unwrap().display_name, "Selected");
    assert_eq!(scope.source_engine.engine_id(), Some("local"));
    assert!(scope.selected_project().unwrap().can_build());
}

#[test]
fn stale_selected_project_does_not_fallback_to_latest_recent() {
    let projects = vec![RecentProject::fixture("Latest", "E:/Projects/Latest", 20)];
    let scope = HubScope::resolve(
        Some(Path::new("E:/Projects/Missing")),
        &projects,
        &ProjectMetadataMap::new(),
        &[],
        None,
    );

    assert!(scope.has_stale_selected_project());
    assert!(scope.selected_or_latest_project().is_none());
}

#[test]
fn no_selection_uses_latest_recent_and_active_engine_scope() {
    let projects = vec![
        RecentProject::fixture("Old", "E:/Projects/Old", 1),
        RecentProject::fixture("Latest", "E:/Projects/Latest", 20),
    ];
    let engines = vec![engine("first"), engine("active")];

    let scope = HubScope::resolve(
        None,
        &projects,
        &ProjectMetadataMap::new(),
        &engines,
        Some("active"),
    );

    assert_eq!(
        scope.selected_or_latest_project().unwrap().display_name,
        "Latest"
    );
    assert_eq!(scope.source_engine.engine_id(), Some("active"));
}

#[test]
fn selected_project_without_engine_binding_reports_project_unbound() {
    let projects = vec![RecentProject::fixture("Game", "E:/Projects/Game", 20)];

    let scope = HubScope::resolve(
        Some(Path::new("E:/Projects/Game")),
        &projects,
        &ProjectMetadataMap::new(),
        &[engine("active")],
        Some("active"),
    );

    assert_eq!(
        scope.source_engine,
        SourceEngineScope::ProjectUnbound {
            project_name: "Game".to_string()
        }
    );
}

#[test]
fn selected_project_with_missing_engine_reports_unavailable_binding() {
    let projects = vec![RecentProject::fixture("Game", "E:/Projects/Game", 20)];
    let mut metadata = ProjectMetadataMap::new();
    metadata_for_path_mut(&mut metadata, "E:/Projects/Game").engine_id =
        Some("missing".to_string());

    let scope = HubScope::resolve(
        Some(Path::new("E:/Projects/Game")),
        &projects,
        &metadata,
        &[engine("active")],
        Some("active"),
    );

    assert_eq!(
        scope.source_engine,
        SourceEngineScope::ProjectEngineUnavailable {
            project_name: "Game".to_string(),
            engine_id: "missing".to_string()
        }
    );
    assert!(!scope.selected_project().unwrap().can_build());
}

#[test]
fn active_engine_scope_falls_back_to_first_engine_then_none() {
    let projects = vec![RecentProject::fixture("Latest", "E:/Projects/Latest", 20)];

    let first_fallback = HubScope::resolve(
        None,
        &projects,
        &ProjectMetadataMap::new(),
        &[engine("first"), engine("second")],
        Some("missing"),
    );
    let no_engine = HubScope::resolve(
        None,
        &projects,
        &ProjectMetadataMap::new(),
        &[],
        Some("missing"),
    );

    assert_eq!(first_fallback.source_engine.engine_id(), Some("first"));
    assert_eq!(no_engine.source_engine, SourceEngineScope::None);
}
