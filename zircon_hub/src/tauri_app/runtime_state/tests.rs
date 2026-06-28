use std::fs;

use crate::projects::{project_metadata_key, RecentProject};
use crate::settings::{BuildProfile, HubConfig, HubLanguage};

use super::*;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn create_valid_source_checkout(source_path: &Path) {
    fs::create_dir_all(source_path.join("tools")).unwrap();
    fs::create_dir_all(source_path.join("zircon_runtime")).unwrap();
    fs::write(
        source_path.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source_path.join("tools").join("zircon_build.py"), "").unwrap();
}

#[test]
fn startup_selection_preserves_persisted_stale_project_path() {
    let recent_projects = vec![RecentProject::new("Recent", "E:/Projects/Recent", 30)];

    let selected = startup_selected_project_path(
        Some(Path::new("E:/Projects/Missing")),
        Some(Path::new("E:/Projects/Recent")),
        &recent_projects,
    );

    assert_eq!(selected, Some(PathBuf::from("E:/Projects/Missing")));
}

#[test]
fn load_from_paths_merges_repairs_registers_source_and_persists_runtime_state() {
    let temp = temp_test_dir("zircon-hub-tauri-runtime-load");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let project_path = temp.join("Game");
    let source_path = temp.join("ZirconEngine");
    fs::create_dir_all(&project_path).unwrap();
    create_valid_source_checkout(&source_path);

    let mut config = HubConfig::default();
    config.recent_projects = vec![RecentProject::new("Game", &project_path, 4)];
    config.project_metadata.insert(
        project_metadata_key(&project_path),
        crate::projects::ProjectMetadata {
            pinned: true,
            engine_id: Some("missing-engine".to_string()),
            last_selected_template: None,
        },
    );
    config.settings.default_source_dir = source_path.clone();
    config.settings.default_build_output_dir = temp.join("out");
    config.runtime.selected_project_path = Some(project_path.clone());
    config.save(&config_path).unwrap();
    fs::write(
        &editor_config_path,
        format!(
            r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[]}}}}"#,
            project_path.to_string_lossy().replace('\\', "/")
        ),
    )
    .unwrap();

    let session =
        HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path.clone())
            .expect("Tauri runtime session should load and persist");

    assert_eq!(session.selected_project_path, Some(project_path.clone()));
    assert_eq!(session.config.engines.len(), 1);
    assert_eq!(
        session.config.active_engine_id.as_deref(),
        Some(source_engine_id(&source_path).as_str())
    );
    assert_eq!(
        session
            .config
            .project_metadata
            .get(&project_metadata_key(&project_path))
            .and_then(|metadata| metadata.engine_id.as_deref()),
        None
    );
    let saved = HubConfig::load(&config_path).unwrap();
    assert_eq!(saved.runtime.selected_project_path, Some(project_path));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_action_applies_typed_payload_and_refreshes_source_engine() {
    let temp = temp_test_dir("zircon-hub-tauri-save-settings-payload");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let source_path = temp.join("ZirconEngine");
    let build_output = temp.join("build-output");
    let device_install = temp.join("device-install");
    create_valid_source_checkout(&source_path);
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
        .expect("Tauri runtime session should load");

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": "py",
                    "cargoPath": "cargo",
                    "rustupPath": "rustup",
                    "defaultProjectDir": temp.join("projects").to_string_lossy(),
                    "defaultSourceDir": source_path.to_string_lossy(),
                    "defaultBuildOutputDir": build_output.to_string_lossy(),
                    "defaultDeviceInstallDir": device_install.to_string_lossy(),
                    "buildProfile": "release",
                    "jobs": 3,
                    "language": "English"
                }
            })),
        })
        .expect("save-settings should accept typed settings payload");

    assert_eq!(session.config.settings.build_profile, BuildProfile::Release);
    assert_eq!(session.config.settings.jobs, 3);
    assert_eq!(session.config.settings.language, HubLanguage::English);
    assert_eq!(session.config.settings.default_source_dir, source_path);
    assert_eq!(
        session.config.settings.default_build_output_dir,
        build_output
    );
    assert_eq!(
        session.config.settings.default_device_install_dir,
        device_install
    );
    let expected_engine_id = source_engine_id(&source_path);
    assert_eq!(
        session.config.active_engine_id.as_deref(),
        Some(expected_engine_id.as_str())
    );
    let active_engine = session
        .config
        .engines
        .iter()
        .find(|engine| engine.id == expected_engine_id)
        .expect("payload Source Engine should be registered");
    assert_eq!(active_engine.source_dir, source_path);
    assert_eq!(active_engine.output_dir, build_output);
    assert_eq!(view_model.settings.language, "English");
    assert_eq!(view_model.task_summary.label, "Settings saved");

    let saved = HubConfig::load(&config_path).unwrap();
    assert_eq!(saved.settings.build_profile, BuildProfile::Release);
    assert_eq!(saved.settings.language, HubLanguage::English);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_refreshes_source_scoped_catalogs_in_returned_view_model() {
    let temp = temp_test_dir("zircon-hub-tauri-save-settings-catalogs");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let source_path = temp.join("ZirconEngine");
    let build_output = temp.join("build-output");
    let device_install = temp.join("device-install");
    let asset_path = source_path
        .join("zircon_editor")
        .join("assets")
        .join("icons")
        .join("source-settings-tool.svg");
    let plugin_manifest_path = source_path
        .join("zircon_plugins")
        .join("source_settings_tools")
        .join("plugin.toml");
    let learn_path = source_path
        .join("docs")
        .join("settings")
        .join("source-settings-refresh.md");
    create_valid_source_checkout(&source_path);
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    fs::write(&asset_path, "<svg></svg>").unwrap();
    fs::create_dir_all(plugin_manifest_path.parent().unwrap()).unwrap();
    fs::write(
        &plugin_manifest_path,
        r#"id = "source_settings_tools"
display_name = "Source Settings Tools"
description = "Source plugin loaded after settings save."
category = "editor"
maturity = "stable"
supported_targets = ["editor_host"]

[[modules]]
name = "source.settings"
kind = "editor"
"#,
    )
    .unwrap();
    fs::create_dir_all(learn_path.parent().unwrap()).unwrap();
    fs::write(
        &learn_path,
        "# Source Settings Refresh\n\nSource Engine docs loaded after settings save.\n",
    )
    .unwrap();
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
        .expect("Tauri runtime session should load");

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": "py",
                    "cargoPath": "cargo",
                    "rustupPath": "rustup",
                    "defaultProjectDir": temp.join("projects").to_string_lossy(),
                    "defaultSourceDir": source_path.to_string_lossy(),
                    "defaultBuildOutputDir": build_output.to_string_lossy(),
                    "defaultDeviceInstallDir": device_install.to_string_lossy(),
                    "buildProfile": "debug",
                    "jobs": 2,
                    "language": "English"
                }
            })),
        })
        .expect("save-settings should refresh source-scoped catalogs");

    let expected_engine_id = source_engine_id(&source_path);
    assert_eq!(
        view_model.active_source_engine_id.as_deref(),
        Some(expected_engine_id.as_str())
    );
    let asset_debug = view_model
        .assets
        .iter()
        .map(|asset| format!("{}:{}", asset.name, asset.source_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.assets.iter().any(|asset| {
            asset.name == "source-settings-tool.svg" && asset.source_key == "engine"
        }),
        "assets should include refreshed Source Engine asset, got {asset_debug:?}"
    );
    let plugin_debug = view_model
        .plugins
        .iter()
        .map(|plugin| format!("{}:{}", plugin.id, plugin.scope_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.plugins.iter().any(|plugin| {
            plugin.id == "source_settings_tools"
                && plugin.scope_key == "engine"
                && plugin.editor_scoped
        }),
        "plugins should include refreshed Source Engine plugin, got {plugin_debug:?}"
    );
    let learn_debug = view_model
        .learn_resources
        .iter()
        .map(|resource| format!("{}:{}", resource.title, resource.source_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.learn_resources.iter().any(|resource| {
            resource.title == "Source Settings Refresh" && resource.source_key == "engine"
        }),
        "learn resources should include refreshed Source Engine doc, got {learn_debug:?}"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn apply_action_records_payload_validation_failure_as_recoverable_status() {
    let temp = temp_test_dir("zircon-hub-tauri-payload-validation-status");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let mut config = HubConfig::default();
    config.settings.language = HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
        .expect("Tauri runtime session should load");

    let model = session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": "projects/Game",
                "template": "renderable-empty"
            })),
        })
        .expect("payload validation failures should return refreshed Hub state");

    assert_eq!(model.task_summary.label, "操作失败");
    assert_eq!(
        model.task_summary.detail,
        "项目位置必须是绝对路径：projects/Game"
    );
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("检查操作 payload 后从 Hub 重试")
    );
    assert_eq!(model.task_summary.tone, "error");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn persist_failure_sets_recoverable_status_and_recovers_after_retry() {
    let temp = temp_test_dir("zircon-hub-tauri-persist-failure");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    HubConfig::default().save(&config_path).unwrap();
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
        .expect("Tauri runtime session should load");
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");

    let error = session
        .apply_action(HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("settings".to_string()),
            payload: None,
        })
        .expect_err("blocked config parent should fail persist");

    assert!(error.to_string().contains("I/O error"));
    assert_eq!(session.task_status.label, "Save Hub state failed");
    assert_eq!(
        session
            .task_status
            .recovery
            .as_ref()
            .map(|message| message.render(HubLanguage::English))
            .as_deref(),
        Some("Check the Hub config path and retry the action")
    );

    session.config_path = config_path.clone();
    let model = session
        .apply_action(HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("projects".to_string()),
            payload: None,
        })
        .expect("restored config path should persist again");

    assert_eq!(model.active_page, "projects");
    assert_eq!(
        HubConfig::load(&config_path).unwrap().runtime.selected_page,
        HubPage::Projects
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_view_action_status_localizes_in_chinese_view_model() {
    let temp = temp_test_dir("zircon-hub-tauri-project-view-localized");
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let mut config = HubConfig::default();
    config.settings.language = HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
        .expect("Tauri runtime session should load");

    let filter_model = session
        .apply_action(HubActionRequest {
            action_id: "set-project-filter".to_string(),
            target_id: Some("missing".to_string()),
            payload: None,
        })
        .expect("project filter action should return refreshed state");
    assert_eq!(filter_model.task_summary.label, "项目已筛选");
    assert_eq!(filter_model.task_summary.detail, "显示缺失项目");

    let sort_model = session
        .apply_action(HubActionRequest {
            action_id: "set-project-sort".to_string(),
            target_id: Some("name".to_string()),
            payload: None,
        })
        .expect("project sort action should return refreshed state");
    assert_eq!(sort_model.task_summary.label, "项目已排序");
    assert_eq!(sort_model.task_summary.detail, "按名称排序");

    let all_model = session
        .apply_action(HubActionRequest {
            action_id: "view-all-projects".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("view-all-projects action should return refreshed state");
    assert_eq!(all_model.task_summary.label, "全部项目");
    assert_eq!(all_model.task_summary.detail, "显示全部最近项目");

    fs::remove_dir_all(temp).unwrap();
}
