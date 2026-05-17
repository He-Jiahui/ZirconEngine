use std::path::PathBuf;

use crate::error::HubError;
use crate::process::{pick_folder, FolderPickerRequest};
use crate::state::TaskStatus;

use super::super::HubWindow;
use super::HubRuntime;

impl HubRuntime {
    pub(super) fn browse_folder(&mut self, ui: &HubWindow, target: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let Some(selection) = pick_folder(&FolderPickerRequest::new(
            folder_picker_title(target),
            self.folder_picker_initial_dir(ui, target),
        ))?
        else {
            self.task_status = TaskStatus {
                label: "Browse cancelled".to_string(),
                detail: folder_picker_title(target).to_string(),
                running: false,
            };
            return Ok(());
        };

        let selected = selection.to_string_lossy().into_owned();
        match target {
            "project-root" => ui.set_project_path(selected.clone().into()),
            "project-location" => {
                self.config.settings.default_project_dir = selection;
                ui.set_project_location(selected.clone().into());
            }
            "source" => {
                self.config.settings.default_source_dir = selection;
                ui.set_source_path(selected.clone().into());
                self.register_source_engine_from_settings();
                self.refresh_asset_catalog()?;
                self.refresh_learn_catalog()?;
                self.refresh_plugin_catalog()?;
                self.refresh_team_overview()?;
            }
            "output" => {
                self.config.settings.default_build_output_dir = selection;
                ui.set_output_path(selected.clone().into());
                self.register_source_engine_from_settings();
            }
            "device-install" => {
                self.config.settings.default_device_install_dir = selection;
                ui.set_device_install_path(selected.clone().into());
            }
            _ => {
                return Err(HubError::message(format!(
                    "Unknown folder browse target: {target}"
                )))
            }
        }
        self.task_status = TaskStatus {
            label: "Folder selected".to_string(),
            detail: selected,
            running: false,
        };
        Ok(())
    }

    fn folder_picker_initial_dir(&self, ui: &HubWindow, target: &str) -> Option<PathBuf> {
        match target {
            "project-root" => first_existing_dir([
                PathBuf::from(ui.get_project_path().to_string()),
                self.config.settings.default_project_dir.clone(),
            ]),
            "project-location" => {
                first_existing_dir([self.config.settings.default_project_dir.clone()])
            }
            "source" => first_existing_dir([self.config.settings.default_source_dir.clone()]),
            "output" => first_existing_dir([self.config.settings.default_build_output_dir.clone()]),
            "device-install" => {
                first_existing_dir([self.config.settings.default_device_install_dir.clone()])
            }
            _ => None,
        }
    }
}

fn folder_picker_title(target: &str) -> &'static str {
    match target {
        "project-root" => "Select project root",
        "project-location" => "Select default project location",
        "source" => "Select Zircon source checkout",
        "output" => "Select staged build output",
        "device-install" => "Select local device install folder",
        _ => "Select folder",
    }
}

fn first_existing_dir<const N: usize>(candidates: [PathBuf; N]) -> Option<PathBuf> {
    candidates
        .into_iter()
        .find(|candidate| !candidate.as_os_str().is_empty() && candidate.is_dir())
}
