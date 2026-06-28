use std::path::PathBuf;

use crate::error::HubError;
use crate::process::FolderPickerRequest;
use crate::settings::HubSettings;
use crate::state::{HubMessage, HubMessageId, SettingsMessageId, TaskOperationKind, TaskStatus};
use crate::tauri_app::action_request::BrowseSettingsFolderPayload;
use crate::tauri_app::view_model::{HubSettingsPayload, HubTextBundle};

use super::HubRuntimeSession;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsFolderField {
    DefaultProjectDir,
    DefaultSourceDir,
    DefaultBuildOutputDir,
    DefaultDeviceInstallDir,
}

impl SettingsFolderField {
    fn from_id(value: &str) -> Option<Self> {
        match value.trim() {
            "defaultProjectDir" | "default-project-dir" | "project-dir" => {
                Some(Self::DefaultProjectDir)
            }
            "defaultSourceDir" | "default-source-dir" | "source-dir" => {
                Some(Self::DefaultSourceDir)
            }
            "defaultBuildOutputDir" | "default-build-output-dir" | "build-output" => {
                Some(Self::DefaultBuildOutputDir)
            }
            "defaultDeviceInstallDir" | "default-device-install-dir" | "device-install" => {
                Some(Self::DefaultDeviceInstallDir)
            }
            _ => None,
        }
    }

    fn current_path(self, settings: &HubSettings) -> PathBuf {
        match self {
            Self::DefaultProjectDir => settings.default_project_dir.clone(),
            Self::DefaultSourceDir => settings.default_source_dir.clone(),
            Self::DefaultBuildOutputDir => settings.default_build_output_dir.clone(),
            Self::DefaultDeviceInstallDir => settings.default_device_install_dir.clone(),
        }
    }

    fn set_path(self, settings: &mut HubSettings, path: PathBuf) {
        match self {
            Self::DefaultProjectDir => settings.default_project_dir = path,
            Self::DefaultSourceDir => settings.default_source_dir = path,
            Self::DefaultBuildOutputDir => settings.default_build_output_dir = path,
            Self::DefaultDeviceInstallDir => settings.default_device_install_dir = path,
        }
    }

    fn label(self, text: HubTextBundle) -> &'static str {
        match self {
            Self::DefaultProjectDir => text.pair("Default Project Directory", "默认项目目录"),
            Self::DefaultSourceDir => text.pair("Default Source Directory", "默认源码目录"),
            Self::DefaultBuildOutputDir => {
                text.pair("Default Build Output Directory", "默认构建输出目录")
            }
            Self::DefaultDeviceInstallDir => {
                text.pair("Default Device Install Directory", "默认设备安装目录")
            }
        }
    }

    fn picker_title(self, text: HubTextBundle) -> &'static str {
        match self {
            Self::DefaultProjectDir => {
                text.pair("Choose Default Project Directory", "选择默认项目目录")
            }
            Self::DefaultSourceDir => {
                text.pair("Choose Default Source Directory", "选择默认源码目录")
            }
            Self::DefaultBuildOutputDir => text.pair(
                "Choose Default Build Output Directory",
                "选择默认构建输出目录",
            ),
            Self::DefaultDeviceInstallDir => text.pair(
                "Choose Default Device Install Directory",
                "选择默认设备安装目录",
            ),
        }
    }
}

impl HubRuntimeSession {
    pub(super) fn update_settings_draft_from_action(
        &mut self,
        settings_payload: HubSettingsPayload,
    ) -> Result<(), HubError> {
        let mut draft = self.settings_draft.clone();
        settings_payload.apply_to_draft(&mut draft)?;
        self.settings_draft = draft;
        Ok(())
    }

    pub(super) fn save_settings_from_action(
        &mut self,
        settings_payload: Option<HubSettingsPayload>,
    ) -> Result<(), HubError> {
        self.save_settings(settings_payload)
    }

    pub(super) fn discard_settings_draft(&mut self) {
        self.settings_draft = self.config.settings.clone();
        self.task_status = TaskStatus::success(
            "Settings draft discarded",
            HubMessage::new(HubMessageId::Settings(
                SettingsMessageId::DraftRestoredSaved,
            )),
        )
        .with_operation(TaskOperationKind::Settings, "Hub settings");
    }

    pub(super) fn restore_default_settings(&mut self) {
        self.settings_draft = HubSettings::default();
        self.task_status = TaskStatus::success(
            "Default settings restored",
            HubMessage::new(HubMessageId::Settings(
                SettingsMessageId::DraftRestoredDefaults,
            )),
        )
        .with_operation(TaskOperationKind::Settings, "Hub settings");
    }

    pub(super) fn browse_settings_folder(
        &mut self,
        target_id: Option<&str>,
        payload: Option<BrowseSettingsFolderPayload>,
    ) -> Result<(), HubError> {
        let mut draft = self.settings_draft.clone();
        if let Some(settings_payload) = payload
            .as_ref()
            .and_then(|payload| payload.settings.clone())
        {
            if let Err(error) = settings_payload.apply_to_draft(&mut draft) {
                self.record_settings_folder_failure(error.into_status_messages().0);
                return Ok(());
            }
        }

        let field = match settings_folder_field_from_target(target_id, payload.as_ref()) {
            Ok(field) => field,
            Err(error) => {
                self.record_settings_folder_failure(error.into_status_messages().0);
                return Ok(());
            }
        };
        let initial_dir = payload
            .as_ref()
            .and_then(|payload| payload.initial_dir.clone())
            .unwrap_or_else(|| field.current_path(&draft));

        let text = HubTextBundle::new(draft.language);
        match (self.folder_picker)(&FolderPickerRequest::new(
            field.picker_title(text),
            Some(initial_dir),
        )) {
            Ok(Some(path)) => {
                field.set_path(&mut draft, path.clone());
                self.settings_draft = draft;
                let text = HubTextBundle::new(self.settings_draft.language);
                self.task_status = TaskStatus::success(
                    text.status_label("Folder selected"),
                    HubMessage::raw_text(path.to_string_lossy().into_owned()),
                )
                .with_operation(TaskOperationKind::Settings, field.label(text));
            }
            Ok(None) => {
                let text = HubTextBundle::new(draft.language);
                self.task_status = TaskStatus::warning(
                    text.status_label("Folder selection cancelled"),
                    HubMessage::new(HubMessageId::Settings(SettingsMessageId::NoFolderSelected)),
                    HubMessage::new(HubMessageId::Settings(
                        SettingsMessageId::ChooseFolderOrKeepCurrent,
                    )),
                )
                .with_operation(TaskOperationKind::Settings, field.label(text));
            }
            Err(error) => {
                self.record_settings_folder_failure(HubMessage::raw_text(error.to_string()))
            }
        }
        Ok(())
    }

    pub(super) fn record_settings_save_failure(&mut self, detail: HubMessage) {
        let text = HubTextBundle::new(self.settings_draft.language);
        self.task_status = TaskStatus::error(
            text.status_label("Save Settings failed"),
            detail,
            HubMessage::new(HubMessageId::Settings(
                SettingsMessageId::CheckValuesAndSave,
            )),
        )
        .with_operation(
            TaskOperationKind::Settings,
            text.pair("Hub settings", "Hub 设置"),
        );
    }

    fn record_settings_folder_failure(&mut self, detail: HubMessage) {
        let text = HubTextBundle::new(self.settings_draft.language);
        self.task_status = TaskStatus::error(
            text.status_label("Browse folder failed"),
            detail,
            HubMessage::new(HubMessageId::Settings(
                SettingsMessageId::ChooseExistingFolderOrManual,
            )),
        )
        .with_operation(
            TaskOperationKind::Settings,
            text.pair("Settings folder", "设置文件夹"),
        );
    }
}

fn settings_folder_field_from_target(
    target_id: Option<&str>,
    payload: Option<&BrowseSettingsFolderPayload>,
) -> Result<SettingsFolderField, HubError> {
    let field_id = target_id
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .map(str::to_string)
        .or_else(|| payload.and_then(|payload| payload.field.clone()))
        .ok_or_else(|| {
            HubError::status(
                HubMessage::new(HubMessageId::Settings(
                    SettingsMessageId::FolderFieldRequired,
                )),
                None,
            )
        })?;

    SettingsFolderField::from_id(&field_id).ok_or_else(|| {
        HubError::status(
            HubMessage::with_params(
                HubMessageId::Settings(SettingsMessageId::UnknownFolderField),
                [field_id],
            ),
            None,
        )
    })
}

#[cfg(test)]
mod tests {
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
        let editor_config_path = temp.join("editor.json");
        let selected_output = temp.join("selected-output");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("persisted-output");
        config.settings.default_source_dir = create_valid_source_checkout(&temp);
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();

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
                payload: None,
            })
            .expect("save-settings without payload should persist settings draft");

        assert_eq!(
            session.config.settings.default_build_output_dir,
            selected_output
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn update_settings_draft_recomputes_health_without_persisting() {
        let temp = temp_test_dir("zircon-hub-settings-draft-health");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.python_path = "python".to_string();
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();

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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = create_valid_source_checkout(&temp);
        config.settings.python_path = "python".to_string();
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();
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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = create_valid_source_checkout(&temp);
        config.settings.python_path = "custom-python".to_string();
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();

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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = create_valid_source_checkout(&temp);
        config.settings.default_project_dir = temp.join("persisted-projects");
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();
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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = create_valid_source_checkout(&temp);
        config.settings.python_path = "python".to_string();
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();
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
        let editor_config_path = temp.join("editor.json");
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
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path).unwrap();

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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.language = crate::settings::HubLanguage::Chinese;
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session = HubRuntimeSession::load_from_paths(config_path, editor_config_path)
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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.language = crate::settings::HubLanguage::Chinese;
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
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
}
