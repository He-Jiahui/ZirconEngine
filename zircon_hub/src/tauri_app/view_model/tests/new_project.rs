use std::path::PathBuf;

use crate::{
    engines::SourceEngineInstall,
    settings::HubSettings,
    state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    },
    team::TeamOverview,
};

use super::*;

#[test]
fn draft_projects_runtime_name_location_template_and_engine() {
    let snapshot = test_snapshot();

    let draft = new_project_draft(&snapshot);

    assert_eq!(draft.name, "Draft Game");
    assert_eq!(draft.location, "E:/Drafts");
    assert_eq!(draft.template, "renderable-empty");
    assert_eq!(draft.engine_id.as_deref(), Some("engine-b"));
}

#[test]
fn draft_falls_back_when_persisted_template_or_engine_is_stale() {
    let mut snapshot = test_snapshot();
    snapshot.selected_template_id = "missing-template".to_string();
    snapshot.new_project_engine_id = Some("missing-engine".to_string());

    let draft = new_project_draft(&snapshot);

    assert_eq!(draft.template, "renderable-empty");
    assert_eq!(draft.engine_id.as_deref(), Some("engine-a"));
}

fn test_snapshot() -> HubSnapshot {
    HubSnapshot {
        selected_page: HubPage::Projects,
        project_filter: ProjectFilterMode::All,
        project_sort: ProjectSortMode::LastModified,
        project_view_mode: ProjectViewMode::Grid,
        project_subpage: ProjectSubpage::NewProject,
        search_query: String::new(),
        selected_project_path: None,
        new_project_name: "Draft Game".to_string(),
        selected_template_id: "renderable-empty".to_string(),
        new_project_location: PathBuf::from("E:/Drafts"),
        new_project_engine_id: Some("engine-b".to_string()),
        pending_delete_project_path: None,
        task_status: TaskStatus::idle(),
        queued_background_actions: 0,
        recent_projects: Vec::new(),
        project_metadata: crate::projects::ProjectMetadataMap::new(),
        assets: Vec::new(),
        learn_resources: Vec::new(),
        plugins: Vec::new(),
        team: TeamOverview::empty(),
        action_history: Vec::new(),
        engines: vec![
            SourceEngineInstall {
                id: "engine-a".to_string(),
                display_name: "Engine A".to_string(),
                source_dir: PathBuf::from("E:/EngineA"),
                output_dir: PathBuf::from("E:/EngineA/out"),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            },
            SourceEngineInstall {
                id: "engine-b".to_string(),
                display_name: "Engine B".to_string(),
                source_dir: PathBuf::from("E:/EngineB"),
                output_dir: PathBuf::from("E:/EngineB/out"),
                last_build_unix_ms: None,
                build_history: Vec::new(),
            },
        ],
        active_engine_id: Some("engine-a".to_string()),
        settings: HubSettings::default(),
        settings_draft: HubSettings::default(),
    }
}
