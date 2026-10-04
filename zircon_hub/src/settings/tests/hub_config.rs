use super::*;
use crate::state::HubMessage;

#[test]
fn hub_config_round_trips_settings_and_engines() {
    let mut config = HubConfig::default();
    config.settings.jobs = 4;
    config
        .settings
        .set_default_device_install_from_native_folder_picker(PathBuf::from("E:/zircon-device"));
    config.project_metadata.insert(
        "e:/projects/game".to_string(),
        crate::projects::ProjectMetadata {
            pinned: true,
            engine_id: Some("local".to_string()),
            last_selected_template: Some("renderable-empty".to_string()),
        },
    );
    config.engines.push(SourceEngineInstall {
        id: "local".to_string(),
        display_name: "Local Source".to_string(),
        source_dir: PathBuf::from("E:/Git/ZirconEngine"),
        output_dir: PathBuf::from("E:/zircon-build"),
        last_build_unix_ms: Some(7),
        build_history: Vec::new(),
    });
    config.active_engine_id = Some("local".to_string());
    config.window.width = Some(1320);
    config.window.height = Some(820);
    config.window.maximized = true;
    config.runtime.selected_page = HubPage::Builds;
    config.runtime.project_subpage = ProjectSubpage::ProjectDetail;
    config.runtime.project_filter = ProjectFilterMode::Existing;
    config.runtime.project_sort = ProjectSortMode::Name;
    config.runtime.project_view_mode = ProjectViewMode::List;
    config.runtime.search_query = "elysium".to_string();
    config.runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Game"));
    config.runtime.new_project_name = "Draft Game".to_string();
    config.runtime.selected_template_id = "renderable-empty".to_string();
    config.runtime.new_project_location = PathBuf::from("E:/Drafts");
    config.runtime.new_project_engine_id = Some("local".to_string());
    config.action_history.push(HubActionRecord {
        finished_unix_ms: 9,
        action: crate::state::HubActionKind::OpenEditor,
        status: crate::state::HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::raw_text("pid 42"),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: Some(42),
        command_line: vec!["zircon_editor".to_string(), "--project".to_string()],
        output_dir: None,
    });

    let encoded = toml::to_string(&config).unwrap();
    let decoded: HubConfig = toml::from_str(&encoded).unwrap();

    assert_eq!(decoded.settings.jobs, 4);
    assert_eq!(
        decoded.settings.default_device_install_dir,
        PathBuf::from("E:/zircon-device")
    );
    assert_eq!(
        decoded.settings.default_device_install_provenance,
        OutputRootProvenance::NativeFolderPicker
    );
    assert_eq!(decoded.engines[0].id, "local");
    assert!(decoded.project_metadata["e:/projects/game"].pinned);
    assert_eq!(
        decoded.project_metadata["e:/projects/game"]
            .engine_id
            .as_deref(),
        Some("local")
    );
    assert_eq!(decoded.active_engine_id.as_deref(), Some("local"));
    assert_eq!(decoded.window.width, Some(1320));
    assert!(decoded.window.maximized);
    assert_eq!(decoded.runtime.selected_page, HubPage::Builds);
    assert_eq!(
        decoded.runtime.project_subpage,
        ProjectSubpage::ProjectDetail
    );
    assert_eq!(decoded.runtime.project_filter, ProjectFilterMode::Existing);
    assert_eq!(decoded.runtime.project_sort, ProjectSortMode::Name);
    assert_eq!(decoded.runtime.project_view_mode, ProjectViewMode::List);
    assert_eq!(decoded.runtime.search_query, "elysium");
    assert_eq!(
        decoded.runtime.selected_project_path,
        Some(PathBuf::from("E:/Projects/Game"))
    );
    assert_eq!(decoded.runtime.new_project_name, "Draft Game");
    assert_eq!(
        decoded.runtime.new_project_engine_id.as_deref(),
        Some("local")
    );
    assert_eq!(decoded.action_history[0].process_id, Some(42));
}

#[test]
fn legacy_settings_without_output_provenance_fail_closed() {
    let config: HubConfig = toml::from_str(
        r#"
                [settings]
                default_build_output_dir = "E:/legacy-builds"
                default_device_install_dir = "E:/legacy-device"
            "#,
    )
    .expect("legacy settings should remain readable");

    assert_eq!(
        config.settings.default_build_output_provenance,
        OutputRootProvenance::Unverified
    );
    assert_eq!(
        config.settings.default_device_install_provenance,
        OutputRootProvenance::Unverified
    );
}

#[test]
fn selecting_the_same_registered_engine_output_preserves_native_provenance() {
    let path = PathBuf::from("E:/picked-builds");
    let mut settings = HubSettings::default();
    settings.set_default_build_output_from_native_folder_picker(path.clone());

    settings.set_default_build_output_from_registered_engine(path);

    assert_eq!(
        settings.default_build_output_provenance,
        OutputRootProvenance::NativeFolderPicker
    );
}

#[test]
fn selecting_a_different_registered_engine_output_clears_provenance() {
    let mut settings = HubSettings::default();
    settings.set_default_build_output_from_native_folder_picker(PathBuf::from("E:/picked-builds"));

    settings.set_default_build_output_from_registered_engine(PathBuf::from("E:/engine-builds"));

    assert_eq!(
        settings.default_build_output_provenance,
        OutputRootProvenance::Unverified
    );
}

#[test]
fn loads_archived_string_action_history_alongside_structured_messages() {
    let text = r#"
[[action_history]]
finished_unix_ms = 2
action = "create-project"
status = "success"
target = "Game"
detail = { id = "project.created-path", params = ["E:/Projects/Game"] }
log_excerpt = ""
command_line = []

[[action_history]]
finished_unix_ms = 1
action = "open-output"
status = "failed"
target = "Old Output"
detail = "archived detail"
log_excerpt = "archived log"
recovery = "archived recovery"
command_line = []
"#;

    let config: HubConfig = toml::from_str(text).unwrap();

    assert_eq!(
        config.action_history[0].detail,
        HubMessage::with_params(
            crate::state::HubMessageId::Project(crate::state::ProjectMessageId::CreatedPath),
            ["E:/Projects/Game"]
        )
    );
    assert_eq!(
        config.action_history[0].detail.render(HubLanguage::Chinese),
        "已创建 E:/Projects/Game"
    );
    assert_eq!(
        config.action_history[1].detail,
        HubMessage::raw_text("archived detail")
    );
    assert_eq!(
        config.action_history[1].recovery.as_ref(),
        Some(&HubMessage::raw_text("archived recovery"))
    );
}

#[test]
fn save_replaces_existing_config_atomically_without_leaving_tmp_file() {
    let temp = temp_test_dir("zircon-hub-config-atomic-replace");
    let path = temp.join("hub.toml");
    let mut first = HubConfig::default();
    first.settings.jobs = 1;
    first.save(&path).unwrap();

    let mut second = HubConfig::default();
    second.settings.jobs = 7;
    second.settings.language = HubLanguage::English;
    second.save(&path).unwrap();

    let loaded = HubConfig::load(&path).unwrap();
    assert_eq!(loaded.settings.jobs, 7);
    assert_eq!(loaded.settings.language, HubLanguage::English);
    assert!(!path.with_extension("toml.tmp").exists());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_keeps_previous_config_when_tmp_write_is_blocked() {
    let temp = temp_test_dir("zircon-hub-config-tmp-blocked");
    let path = temp.join("hub.toml");
    let mut first = HubConfig::default();
    first.settings.jobs = 2;
    first.save(&path).unwrap();
    let original_text = fs::read_to_string(&path).unwrap();
    fs::create_dir(path.with_extension("toml.tmp")).unwrap();

    let mut second = HubConfig::default();
    second.settings.jobs = 9;
    let error = second
        .save(&path)
        .expect_err("tmp path directory should block atomic save");

    assert!(error.to_string().contains("I/O error"));
    assert_eq!(fs::read_to_string(&path).unwrap(), original_text);

    fs::remove_dir_all(temp).unwrap();
}

#[cfg(windows)]
#[test]
fn save_keeps_previous_config_when_atomic_replace_is_denied() {
    let temp = temp_test_dir("zircon-hub-config-replace-denied");
    let path = temp.join("hub.toml");
    let mut first = HubConfig::default();
    first.settings.jobs = 2;
    first.save(&path).unwrap();
    let original_text = fs::read_to_string(&path).unwrap();

    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).unwrap();

    let mut second = HubConfig::default();
    second.settings.jobs = 9;
    let result = second.save(&path);
    let current_text = fs::read_to_string(&path).unwrap();
    let tmp_exists = path.with_extension("toml.tmp").exists();

    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&path, permissions).unwrap();

    let error = result.expect_err("read-only target should reject atomic replacement");
    assert!(error.to_string().contains("atomic replace failed"));
    assert_eq!(current_text, original_text);
    assert!(!tmp_exists);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn runtime_state_normalizes_empty_persisted_inputs() {
    let mut state = HubRuntimeState {
        selected_project_path: Some(PathBuf::new()),
        new_project_name: "  Draft Game  ".to_string(),
        selected_template_id: String::new(),
        new_project_location: PathBuf::new(),
        new_project_engine_id: Some(String::new()),
        ..HubRuntimeState::default()
    };

    state.normalize();

    assert!(state.selected_project_path.is_none());
    assert_eq!(state.new_project_name, "Draft Game");
    assert_eq!(
        state.selected_template_id,
        ProjectTemplateId::RenderableEmpty.as_str()
    );
    assert_eq!(state.new_project_location, default_project_dir());
    assert!(state.new_project_engine_id.is_none());
}

#[test]
fn settings_parse_profile_and_language_from_ui_values() {
    assert_eq!(
        BuildProfile::from_ui_value(" RELEASE "),
        Some(BuildProfile::Release)
    );
    assert_eq!(
        BuildProfile::from_ui_value(" DEBUG "),
        Some(BuildProfile::Debug)
    );
    assert_eq!(BuildProfile::from_ui_value("fast"), None);
    assert_eq!(
        HubLanguage::from_ui_value(" ENGLISH "),
        Some(HubLanguage::English)
    );
    assert_eq!(HubLanguage::from_ui_value("zh"), Some(HubLanguage::Chinese));
    assert_eq!(HubLanguage::from_ui_value("\u{00c9}NGLISH"), None);
    assert_eq!(HubLanguage::English.as_ui_value(), "English");
}

#[test]
fn config_repair_deduplicates_and_prunes_foundation_registries() {
    let mut config = HubConfig::default();
    config.recent_projects = vec![
        RecentProject::fixture("Old", "E:/Projects/Game", 1),
        RecentProject::fixture("New", "E:/Projects/Game", 9),
        RecentProject::fixture("Other", "E:/Projects/Other", 2),
    ];
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Game"),
        crate::projects::ProjectMetadata {
            pinned: true,
            ..crate::projects::ProjectMetadata::default()
        },
    );
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Removed"),
        crate::projects::ProjectMetadata {
            pinned: true,
            ..crate::projects::ProjectMetadata::default()
        },
    );
    config.engines.push(SourceEngineInstall {
        id: "local".to_string(),
        display_name: "Local".to_string(),
        source_dir: PathBuf::from("E:/src"),
        output_dir: PathBuf::from("E:/out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some("missing".to_string());

    let report = config.repair_registries();

    assert!(report.repaired_anything());
    assert_eq!(config.recent_projects.len(), 2);
    assert_eq!(config.recent_projects[0].summary.name, "New");
    assert!(config
        .project_metadata
        .contains_key(&project_metadata_key("E:/Projects/Game")));
    assert!(!config
        .project_metadata
        .contains_key(&project_metadata_key("E:/Projects/Removed")));
    assert_eq!(config.active_engine_id.as_deref(), Some("local"));
}

#[test]
fn config_repair_truncates_recent_projects_and_action_history() {
    let mut config = HubConfig::default();
    config.recent_projects = (0..(RECENT_PROJECT_LIMIT + 3))
        .map(|index| {
            RecentProject::fixture(
                format!("Project {index}"),
                format!("E:/Projects/{index}"),
                index as u64,
            )
        })
        .collect();
    config.action_history = (0..(ACTION_HISTORY_LIMIT + 2))
        .map(|index| HubActionRecord {
            finished_unix_ms: index as u64,
            action: crate::state::HubActionKind::OpenEditor,
            status: crate::state::HubActionStatus::Success,
            target: format!("Project {index}"),
            detail: HubMessage::empty(),
            log_excerpt: HubMessage::empty(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: None,
        })
        .collect();

    let report = config.repair_registries();

    assert_eq!(config.recent_projects.len(), RECENT_PROJECT_LIMIT);
    assert_eq!(config.recent_projects[0].summary.name, "Project 10");
    assert_eq!(config.action_history.len(), ACTION_HISTORY_LIMIT);
    assert_eq!(config.action_history[0].finished_unix_ms, 17);
    assert_eq!(report.removed_recent_projects, 3);
    assert_eq!(report.removed_action_history, 2);
}

#[test]
fn config_repair_preserves_valid_metadata_and_active_engine() {
    let mut config = HubConfig::default();
    config.recent_projects = vec![RecentProject::fixture("Game", "E:/Projects/Game", 1)];
    config.project_metadata.insert(
        project_metadata_key("E:/Projects/Game"),
        crate::projects::ProjectMetadata {
            pinned: true,
            engine_id: Some("local".to_string()),
            last_selected_template: Some("renderable-empty".to_string()),
        },
    );
    config.engines.push(SourceEngineInstall {
        id: "local".to_string(),
        display_name: "Local".to_string(),
        source_dir: PathBuf::from("E:/src"),
        output_dir: PathBuf::from("E:/out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some("local".to_string());

    let report = config.repair_registries();

    assert!(!report.repaired_anything());
    let metadata = &config.project_metadata[&project_metadata_key("E:/Projects/Game")];
    assert!(metadata.pinned);
    assert_eq!(metadata.engine_id.as_deref(), Some("local"));
    assert_eq!(
        metadata.last_selected_template.as_deref(),
        Some("renderable-empty")
    );
    assert_eq!(config.active_engine_id.as_deref(), Some("local"));
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
