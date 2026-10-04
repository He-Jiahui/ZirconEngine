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
            Self::DefaultBuildOutputDir => {
                settings.set_default_build_output_from_native_folder_picker(path)
            }
            Self::DefaultDeviceInstallDir => {
                settings.set_default_device_install_from_native_folder_picker(path)
            }
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
#[path = "tests/settings_actions.rs"]
mod tests;
