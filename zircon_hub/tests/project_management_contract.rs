use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use zircon_hub::projects::{
    metadata_for_path, metadata_for_path_mut, project_metadata_key, project_paths_match,
    project_template_catalog, prune_empty_metadata, save_editor_recent_projects,
    CreateProjectRequest, ProjectMetadata, ProjectMetadataMap, ProjectTemplate, RecentProject,
    RecycleDeleteCommand,
};
use zircon_hub::settings::{HubConfig, HubSettings};
use zircon_hub::state::{
    HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode,
    TaskStatus,
};
use zircon_hub::team::TeamOverview;

#[test]
fn project_metadata_uses_normalized_project_paths() {
    let mut metadata = ProjectMetadataMap::new();
    metadata_for_path_mut(&mut metadata, "E:\\Projects\\Game\\").pinned = true;

    assert_eq!(
        project_metadata_key("E:\\Projects\\Game\\"),
        "e:/projects/game"
    );
    assert!(project_paths_match(
        "E:\\Projects\\Game\\",
        "e:/projects/game/"
    ));
    assert!(
        metadata_for_path(&metadata, "E:/Projects/Game")
            .unwrap()
            .pinned
    );

    metadata_for_path_mut(&mut metadata, "E:/Projects/Game").pinned = false;
    prune_empty_metadata(&mut metadata);

    assert!(metadata.is_empty());
}

#[test]
fn hub_project_metadata_round_trips_in_hub_config() {
    let mut config = HubConfig::default();
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Game"),
        ProjectMetadata {
            pinned: true,
            engine_id: Some("local-source".to_string()),
            last_selected_template: Some("renderable-empty".to_string()),
        },
    );

    let encoded = toml::to_string(&config).unwrap();
    let decoded: HubConfig = toml::from_str(&encoded).unwrap();
    let metadata = metadata_for_path(&decoded.project_metadata, "e:/projects/game").unwrap();

    assert!(metadata.pinned);
    assert_eq!(metadata.engine_id.as_deref(), Some("local-source"));
    assert_eq!(
        metadata.last_selected_template.as_deref(),
        Some("renderable-empty")
    );
}

#[test]
fn editor_recent_writer_keeps_hub_metadata_out_of_editor_json() {
    let root = temp_test_dir("zircon_hub_recent_metadata_contract");
    let path = root.join("config.json");

    save_editor_recent_projects(&path, &[RecentProject::new("Game", "E:/Projects/Game", 42)])
        .unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(!text.contains("pinned"));
    assert!(!text.contains("engine_id"));
    assert!(!text.contains("last_selected_template"));

    let values = serde_json::from_str::<HashMap<String, Value>>(&text).unwrap();
    assert_eq!(
        values["editor.startup.session"]["recent_projects"][0]["path"],
        Value::String("E:/Projects/Game".to_string())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn new_project_creation_only_accepts_enabled_templates() {
    let enabled_templates = project_template_catalog()
        .iter()
        .filter(|template| template.enabled)
        .map(|template| template.id)
        .collect::<Vec<_>>();

    assert_eq!(enabled_templates, vec!["renderable-empty"]);
    assert_eq!(
        ProjectTemplate::from_enabled_id("renderable-empty"),
        Some(ProjectTemplate::RenderableEmpty)
    );
    assert_eq!(ProjectTemplate::from_enabled_id("3d-scene"), None);

    let request = CreateProjectRequest::new(
        "Demo",
        "E:/Projects",
        ProjectTemplate::from_enabled_id("renderable-empty").unwrap(),
    );

    assert_eq!(request.template.as_editor_arg(), "renderable-empty");
}

#[test]
fn recycle_delete_command_targets_windows_recycle_bin_without_deleting() {
    if !cfg!(target_os = "windows") {
        assert!(RecycleDeleteCommand::for_project("E:/Projects/Game").is_err());
        return;
    }

    let command = RecycleDeleteCommand::for_project("E:/Projects/My Game").unwrap();

    assert_eq!(command.program, "powershell");
    assert!(command.args.iter().any(|arg| arg == "-STA"));
    assert!(command.args[3].contains("Microsoft.VisualBasic.FileIO.FileSystem"));
    assert!(command.args[3].contains("SendToRecycleBin"));
    assert!(command.args[3].contains("E:/Projects/My Game"));
    assert!(RecycleDeleteCommand::for_project("").is_err());
}

#[test]
fn recycle_delete_command_escapes_single_quotes() {
    if !cfg!(target_os = "windows") {
        return;
    }

    let command = RecycleDeleteCommand::for_project("E:/Projects/Designer's Game").unwrap();

    assert!(command.args[3].contains("Designer''s Game"));
}

#[test]
fn filtered_recent_projects_applies_search_filter_and_sort_order() {
    let snapshot = test_snapshot(vec![
        RecentProject::new("Zeta", "E:/Projects/Zeta", 30),
        RecentProject::new("Alpha", "E:/Projects/Alpha", 10),
        RecentProject::new("Arcade", "E:/Arcade/Game", 20),
    ])
    .with_sort(ProjectSortMode::Name)
    .with_query("project")
    .build();

    let projects = snapshot.filtered_recent_projects();

    assert_eq!(
        projects
            .iter()
            .map(|project| project.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha", "Zeta"]
    );
}

#[test]
fn filtered_recent_projects_respects_existing_missing_filter() {
    let root = temp_test_dir("zircon_hub_existing_filter_contract");
    let existing = root.join("Existing");
    let missing = root.join("Missing");
    fs::create_dir_all(&existing).unwrap();

    let snapshot = test_snapshot(vec![
        RecentProject::new("Missing", missing, 30),
        RecentProject::new("Existing", existing, 10),
    ])
    .with_filter(ProjectFilterMode::Existing)
    .build();

    let projects = snapshot.filtered_recent_projects();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].display_name, "Existing");

    let _ = fs::remove_dir_all(root);
}

struct SnapshotBuilder {
    snapshot: HubSnapshot,
}

impl SnapshotBuilder {
    fn with_sort(mut self, project_sort: ProjectSortMode) -> Self {
        self.snapshot.project_sort = project_sort;
        self
    }

    fn with_filter(mut self, project_filter: ProjectFilterMode) -> Self {
        self.snapshot.project_filter = project_filter;
        self
    }

    fn with_query(mut self, query: &str) -> Self {
        self.snapshot.search_query = query.to_string();
        self
    }

    fn build(self) -> HubSnapshot {
        self.snapshot
    }
}

fn test_snapshot(recent_projects: Vec<RecentProject>) -> SnapshotBuilder {
    SnapshotBuilder {
        snapshot: HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects,
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        },
    }
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{prefix}_{}", unique_suffix()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
