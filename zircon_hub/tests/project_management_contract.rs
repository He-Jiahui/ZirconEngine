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
    HubActionKind, HubActionRecord, HubActionStatus, HubPage, HubSnapshot, ProjectFilterMode,
    ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
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
fn hub_config_repair_converges_foundation_registries() {
    let config = fs::read_to_string(crate_dir().join("src/settings/hub_config.rs"))
        .expect("hub_config.rs should be readable");

    for snippet in [
        "pub fn repair_registries(&mut self) -> HubConfigRepairReport",
        "deduplicate_recent_projects(&mut self.recent_projects)",
        "prune_unowned_project_metadata(",
        "truncate_action_history(&mut self.action_history)",
        "repair_active_engine(",
        "fn config_repair_deduplicates_and_prunes_foundation_registries()",
    ] {
        assert!(
            config.contains(snippet),
            "HubConfig should repair persisted project, metadata, engine, and action-history registries; missing {snippet}"
        );
    }

    let runtime = fs::read_to_string(crate_dir().join("src/app/runtime/persistence.rs"))
        .expect("runtime/persistence.rs should be readable");
    assert!(
        runtime.matches("config.repair_registries();").count() >= 2,
        "HubRuntime::load should repair merged editor/Hub config before projecting snapshots and again after Source Engine registration"
    );
}

#[test]
fn runtime_persistence_keeps_editor_recent_writes_in_persistence_owner() {
    let runtime = fs::read_to_string(crate_dir().join("src/app/runtime.rs"))
        .expect("runtime.rs should be readable");
    let workspace = fs::read_to_string(crate_dir().join("src/app/runtime/project_workspace.rs"))
        .expect("project_workspace.rs should be readable");
    let persistence = fs::read_to_string(crate_dir().join("src/app/runtime/persistence.rs"))
        .expect("runtime/persistence.rs should be readable");

    for (name, source) in [("runtime.rs", runtime), ("project_workspace.rs", workspace)] {
        assert!(
            !source.contains("save_editor_recent_projects"),
            "{name} must not write Editor recent JSON directly; route through runtime/persistence.rs"
        );
    }

    for snippet in [
        "pub(super) fn persist(&self) -> Result<(), HubError>",
        "pub(super) fn persist_hub_config(&self) -> Result<(), HubError>",
        "pub(super) fn persist_with_last_project(",
        "save_editor_recent_projects_with_last_project(",
        "save_editor_recent_projects(&self.editor_config_path, &self.config.recent_projects)",
    ] {
        assert!(
            persistence.contains(snippet),
            "runtime/persistence.rs should own Hub TOML and Editor recent save boundaries; missing {snippet}"
        );
    }
}

#[test]
fn foundation_docs_reference_final_gate_plan_and_contracts() {
    let foundations =
        fs::read_to_string(crate_dir().join("../docs/zircon_hub/state/foundations.md"))
            .expect("docs/zircon_hub/state/foundations.md should be readable");

    for snippet in [
        "related_code:",
        "implementation_files:",
        "plan_sources:",
        "tests:",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-foundations-contracts-docs/plan.md",
        ".opencode/workflows/20260528_190023_866_继续完善hub/hub-foundations-contracts-docs/decomposition.md",
        "zircon_hub/tests/project_management_contract.rs",
        "zircon_hub/tests/project_quick_actions_contract.rs",
        "zircon_hub/tests/project_source_engine_contract.rs",
        "zircon_hub/tests/ui_selected_project_runtime_contract.rs",
        "cargo test -p zircon_hub --test project_management_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test project_quick_actions_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test project_source_engine_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture",
        "## Foundation contract gate",
    ] {
        assert!(
            foundations.contains(snippet),
            "Foundation docs should record final-gate plans, contracts, and validation evidence; missing {snippet}"
        );
    }
}

#[test]
fn hub_config_repair_normalizes_registries_before_projection() {
    let mut config = HubConfig::default();
    config.recent_projects = vec![
        RecentProject::new("Older duplicate", "E:/Projects/Game", 10),
        RecentProject::new("Newest duplicate", "E:/Projects/Game", 30),
        RecentProject::new("Other", "E:/Projects/Other", 20),
    ];
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Game"),
        ProjectMetadata {
            pinned: true,
            engine_id: Some("stale-engine".to_string()),
            last_selected_template: Some("renderable-empty".to_string()),
        },
    );
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Removed"),
        ProjectMetadata {
            pinned: true,
            ..ProjectMetadata::default()
        },
    );
    config.active_engine_id = Some("missing-engine".to_string());

    for index in 0..20 {
        config.action_history.push(HubActionRecord {
            finished_unix_ms: index,
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Success,
            target: format!("target {index}"),
            detail: "opened".to_string(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: None,
        });
    }

    let report = config.repair_registries();

    assert!(report.repaired_anything());
    assert_eq!(config.recent_projects.len(), 2);
    assert_eq!(config.recent_projects[0].display_name, "Newest duplicate");
    assert!(metadata_for_path(&config.project_metadata, "E:/Projects/Game").is_some());
    assert!(metadata_for_path(&config.project_metadata, "E:/Projects/Removed").is_none());
    assert_eq!(
        config.action_history.len(),
        zircon_hub::state::ACTION_HISTORY_LIMIT
    );
    assert_eq!(config.active_engine_id, None);
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
        "  Demo  ",
        "E:/Projects",
        ProjectTemplate::from_enabled_id("renderable-empty").unwrap(),
    );

    assert_eq!(request.project_name, "Demo");
    assert_eq!(request.template.as_editor_arg(), "renderable-empty");
    assert_eq!(request.validate_launch_fields(), Ok(()));
    assert_eq!(
        request.target_root(),
        PathBuf::from("E:/Projects").join("Demo")
    );

    let missing_name = CreateProjectRequest::new("   ", "E:/Projects", request.template);
    assert_eq!(
        missing_name.validate_launch_fields(),
        Err("Project name is required")
    );
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
            action_history: Vec::new(),
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

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
