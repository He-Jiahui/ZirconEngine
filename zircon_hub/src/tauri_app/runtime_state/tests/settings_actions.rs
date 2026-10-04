use std::{fs, path::PathBuf};

use crate::settings::HubConfig;
use crate::tauri_app::{action_request::HubAction, HubActionRequest};

use super::*;

#[test]
fn browse_settings_folder_payload_accepts_flat_field_and_initial_dir() {
    let initial_dir = PathBuf::from("E:/Drafts");

    let action = HubActionRequest {
        action_id: "browse-settings-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "field": "defaultProjectDir",
            "initialDir": initial_dir.to_string_lossy(),
            "settings": {
                "defaultProjectDir": "E:/Projects"
            }
        })),
    }
    .parse()
    .expect("browse settings folder action should parse");
    let HubAction::BrowseSettingsFolder { payload, .. } = action else {
        panic!("browse settings folder action should carry folder payload");
    };
    let payload = payload.expect("browse settings folder payload should be present");

    assert_eq!(
        settings_folder_field_from_target(None, Some(&payload)).unwrap(),
        SettingsFolderField::DefaultProjectDir
    );
    assert_eq!(payload.initial_dir, Some(initial_dir));
    assert!(payload.settings.is_some());
}

#[test]
fn settings_draft_folder_changes_wait_for_save_settings() {
    let temp = temp_test_dir("zircon-hub-settings-draft-folder");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let selected_output = temp.join("selected-output");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("persisted-output");
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();

    SettingsFolderField::DefaultBuildOutputDir
        .set_path(&mut session.settings_draft, selected_output.clone());
    let model = session.view_model();

    assert_eq!(
        model.settings.default_build_output_dir,
        temp.join("persisted-output").to_string_lossy().into_owned()
    );
    assert_eq!(
        model.settings_draft.default_build_output_dir,
        selected_output.to_string_lossy().into_owned()
    );
    assert_eq!(
        HubConfig::load(&config_path)
            .unwrap()
            .settings
            .default_build_output_dir,
        temp.join("persisted-output")
    );

    session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "defaultBuildOutputDir": selected_output.to_string_lossy()
                }
            })),
        })
        .expect("save-settings should persist the native-picked draft path");

    assert_eq!(
        session.config.settings.default_build_output_dir,
        selected_output
    );
    assert!(
        session
            .config
            .settings
            .default_build_output_grants_open_capability(),
        "a native folder selection should grant the configured output capability"
    );
    assert!(
        HubConfig::load(&config_path)
            .unwrap()
            .settings
            .default_build_output_grants_open_capability(),
        "native folder provenance must survive a Hub restart"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn typed_settings_output_paths_require_native_selection_before_open_capability() {
    let temp = temp_test_dir("zircon-hub-settings-output-provenance");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let source_dir = create_valid_source_checkout(&temp);
    let build_output = temp.join("webview-build-output");
    let device_install = temp.join("webview-device-install");
    fs::create_dir_all(&build_output).unwrap();
    fs::create_dir_all(&device_install).unwrap();
    let mut config = HubConfig::default();
    config.settings.default_source_dir = source_dir;
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();

    session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "defaultBuildOutputDir": build_output.to_string_lossy(),
                    "defaultDeviceInstallDir": device_install.to_string_lossy()
                }
            })),
        })
        .expect("typed settings should be saved as a recoverable Hub action");

    assert_eq!(
        session.config.settings.default_build_output_dir,
        build_output
    );
    assert_eq!(
        session.config.settings.default_device_install_dir,
        device_install
    );
    assert!(
        !session
            .config
            .settings
            .default_build_output_grants_open_capability(),
        "typed WebView settings must not mint an output-open capability"
    );
    assert!(
        !session
            .config
            .settings
            .default_device_install_grants_open_capability(),
        "typed WebView settings must not mint a device-install open capability"
    );
    let saved = HubConfig::load(&config_path).unwrap();
    assert!(
        !saved.settings.default_build_output_grants_open_capability(),
        "unverified provenance must survive a Hub restart"
    );
    assert!(
        !saved
            .settings
            .default_device_install_grants_open_capability(),
        "unverified device provenance must survive a Hub restart"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn update_settings_draft_recomputes_health_without_persisting() {
    let temp = temp_test_dir("zircon-hub-settings-draft-health");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.python_path = "python".to_string();
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();

    let model = session
        .apply_action(HubActionRequest {
            action_id: "update-settings-draft".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": ""
                }
            })),
        })
        .expect("update-settings-draft should apply to the editable draft");

    assert_eq!(model.settings.python_path, "python");
    assert_eq!(model.settings_draft.python_path, "");
    let python_row = model
        .settings_draft
        .health
        .rows
        .iter()
        .find(|row| row.id == "python-path")
        .expect("Python health row should exist");
    assert_eq!(python_row.state, "error");
    assert_eq!(python_row.meta, "必需");
    assert_eq!(model.settings_draft.health.tone, "warning");
    assert_eq!(
        HubConfig::load(&config_path).unwrap().settings.python_path,
        "python"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn discard_settings_draft_restores_saved_settings_without_persisting() {
    let temp = temp_test_dir("zircon-hub-settings-discard-draft");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.settings.python_path = "python".to_string();
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();
    session.settings_draft.python_path = "broken-python".to_string();

    session
        .apply_action(HubActionRequest {
            action_id: "discard-settings-draft".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("discard-settings-draft should refresh state");

    assert_eq!(session.settings_draft.python_path, "python");
    assert_eq!(
        HubConfig::load(&config_path).unwrap().settings.python_path,
        "python"
    );
    assert_eq!(session.task_status.label, "Settings draft discarded");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn restore_default_settings_updates_draft_without_persisting() {
    let temp = temp_test_dir("zircon-hub-settings-restore-defaults");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.settings.python_path = "custom-python".to_string();
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();

    session
        .apply_action(HubActionRequest {
            action_id: "restore-default-settings".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("restore-default-settings should refresh state");

    assert_eq!(session.settings_draft, HubSettings::default());
    assert_eq!(
        HubConfig::load(&config_path).unwrap().settings.python_path,
        "custom-python"
    );
    assert_eq!(session.task_status.label, "Default settings restored");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn browse_settings_folder_cancel_keeps_existing_draft() {
    let temp = temp_test_dir("zircon-hub-settings-browse-cancel");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.settings.default_project_dir = temp.join("persisted-projects");
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();
    session.folder_picker = |_| Ok(None);

    session
        .apply_action(HubActionRequest {
            action_id: "browse-settings-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "field": "defaultProjectDir",
                "settings": {
                    "defaultProjectDir": temp.join("payload-projects")
                }
            })),
        })
        .expect("cancelled browse should refresh state");

    assert_eq!(
        session.settings_draft.default_project_dir,
        temp.join("persisted-projects")
    );
    assert_eq!(
        HubConfig::load(&config_path)
            .unwrap()
            .settings
            .default_project_dir,
        temp.join("persisted-projects")
    );
    assert_eq!(session.task_status.label, "已取消选择文件夹");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_rejects_invalid_draft_from_shared_field_spec() {
    let temp = temp_test_dir("zircon-hub-settings-save-shared-field-spec");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.settings.python_path = "python".to_string();
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();
    session.settings_draft.python_path.clear();

    session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("invalid draft save should be recoverable");

    assert_eq!(session.task_status.label, "保存设置失败");
    assert_eq!(
        session
            .task_status
            .detail
            .render(crate::settings::HubLanguage::Chinese),
        "需要 Python 可执行文件"
    );
    assert_eq!(
        HubConfig::load(&config_path).unwrap().settings.python_path,
        "python"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_rejects_source_engine_without_runtime_workspace_member() {
    let temp = temp_test_dir("zircon-hub-settings-save-invalid-source");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let invalid_source = temp.join("InvalidSource");
    fs::create_dir_all(invalid_source.join("tools")).unwrap();
    fs::write(
        invalid_source.join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    )
    .unwrap();
    fs::write(invalid_source.join("tools").join("zircon_build.py"), "").unwrap();
    let mut config = HubConfig::default();
    config.settings.default_source_dir = create_valid_source_checkout(&temp);
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .unwrap();

    session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "defaultSourceDir": invalid_source
                }
            })),
        })
        .expect("invalid source checkout should be recoverable");

    assert_eq!(session.task_status.label, "保存设置失败");
    assert_eq!(
        session
            .task_status
            .detail
            .render(crate::settings::HubLanguage::Chinese),
        "源码检出工作区缺少 zircon_runtime 成员"
    );
    assert_ne!(
        HubConfig::load(&config_path)
            .unwrap()
            .settings
            .default_source_dir,
        invalid_source
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn browse_settings_folder_errors_use_current_language() {
    let temp = temp_test_dir("zircon-hub-settings-folder-language");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.language = crate::settings::HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session = HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path)
        .expect("session should load");

    session
        .browse_settings_folder(Some("missing-field"), None)
        .expect("browse folder errors are recoverable");

    assert_eq!(session.task_status.label, "浏览文件夹失败");
    assert_eq!(
        session
            .task_status
            .recovery
            .as_ref()
            .map(|message| message.render(crate::settings::HubLanguage::Chinese))
            .as_deref(),
        Some("选择已有本地文件夹或手动输入路径")
    );
    assert_eq!(session.task_status.target.as_deref(), Some("设置文件夹"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn settings_folder_picker_title_uses_current_language() {
    let chinese = HubTextBundle::new(crate::settings::HubLanguage::Chinese);
    let english = HubTextBundle::new(crate::settings::HubLanguage::English);

    assert_eq!(
        SettingsFolderField::DefaultProjectDir.picker_title(chinese),
        "选择默认项目目录"
    );
    assert_eq!(
        SettingsFolderField::DefaultSourceDir.picker_title(chinese),
        "选择默认源码目录"
    );
    assert_eq!(
        SettingsFolderField::DefaultBuildOutputDir.picker_title(chinese),
        "选择默认构建输出目录"
    );
    assert_eq!(
        SettingsFolderField::DefaultDeviceInstallDir.picker_title(chinese),
        "选择默认设备安装目录"
    );

    assert_eq!(
        SettingsFolderField::DefaultProjectDir.picker_title(english),
        "Choose Default Project Directory"
    );
}

#[test]
fn save_settings_validation_errors_return_localized_view_model() {
    let temp = temp_test_dir("zircon-hub-settings-save-validation");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.language = crate::settings::HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("session should load");

    let model = session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "language": "Klingon"
                }
            })),
        })
        .expect("invalid settings values should be recoverable Hub feedback");

    assert_eq!(model.task_summary.label, "保存设置失败");
    assert_eq!(model.task_summary.detail, "未知 Hub 语言：Klingon");
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("检查设置值后重新保存")
    );
    assert_eq!(model.task_summary.operation, "设置: Hub 设置");
    assert_eq!(
        HubConfig::load(&config_path).unwrap().settings.language,
        crate::settings::HubLanguage::Chinese
    );

    fs::remove_dir_all(temp).unwrap();
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

fn create_valid_source_checkout(temp: &std::path::Path) -> PathBuf {
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(source.join("tools")).unwrap();
    fs::create_dir_all(source.join("zircon_runtime")).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();
    source
}
