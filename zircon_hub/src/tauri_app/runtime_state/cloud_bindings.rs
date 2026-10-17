use std::path::PathBuf;
use zircon_runtime_interface::project::ProjectGuid;

use crate::error::HubError;
use crate::projects::{
    project_filesystem_path_key, project_paths_match, CloudAccountScope, CloudProjectBinding,
    RecentProject,
};

use super::HubRuntimeSession;

const MAX_CLOUD_BINDINGS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tauri_app) struct LocalCloudProjectIdentity {
    pub root: PathBuf,
    pub path_key: String,
    pub guid: ProjectGuid,
}

impl HubRuntimeSession {
    /// Only the explicit selected project can be linked. Re-read its manifest so a
    /// moved or edited tree cannot silently inherit a previous binding.
    pub(in crate::tauri_app) fn selected_local_cloud_project(
        &self,
    ) -> Option<LocalCloudProjectIdentity> {
        let selected = self.snapshot().scope().selected_project()?.path.clone();
        let recent = self
            .config
            .recent_projects
            .iter()
            .find(|item| project_paths_match(&item.path, &selected))?;
        let cached_guid = recent.summary.project_guid?;
        let disk = RecentProject::from_project_path(&selected, recent.last_opened_unix_ms).ok()?;
        let guid = disk.summary.project_guid?;
        if guid != cached_guid {
            return None;
        }
        let root = selected.canonicalize().ok()?;
        if !root.is_dir() {
            return None;
        }
        Some(LocalCloudProjectIdentity {
            path_key: project_filesystem_path_key(&root),
            root,
            guid,
        })
    }

    pub(in crate::tauri_app) fn selected_cloud_binding(
        &self,
        account: &CloudAccountScope,
    ) -> Option<CloudProjectBinding> {
        let selected = self.selected_local_cloud_project()?;
        let mut matches = self.config.cloud_bindings.iter().filter(|binding| {
            binding.is_valid()
                && binding.account == *account
                && binding.local_path_key == selected.path_key
                && binding.local_project_guid == selected.guid
        });
        let binding = matches.next()?.clone();
        matches.next().is_none().then_some(binding)
    }

    /// Revalidate after the service request and save the binding atomically in Hub config.
    pub(in crate::tauri_app) fn attach_cloud_binding(
        &mut self,
        account: CloudAccountScope,
        expected_local: &LocalCloudProjectIdentity,
        organization_id: String,
        project_id: String,
    ) -> Result<CloudProjectBinding, HubError> {
        if self.selected_local_cloud_project().as_ref() != Some(expected_local) {
            return Err(HubError::message("hub_cloud_binding_project_changed"));
        }
        let binding = CloudProjectBinding::new(
            account,
            expected_local.path_key.clone(),
            expected_local.guid,
            organization_id,
            project_id,
        )
        .ok_or_else(|| HubError::message("hub_cloud_binding_invalid"))?;
        let previous = self.config.cloud_bindings.clone();
        self.config.cloud_bindings.retain(|item| {
            item.account != binding.account || item.local_path_key != binding.local_path_key
        });
        if self.config.cloud_bindings.len() >= MAX_CLOUD_BINDINGS {
            self.config.cloud_bindings = previous;
            return Err(HubError::message("hub_cloud_binding_capacity"));
        }
        self.config.cloud_bindings.push(binding.clone());
        if let Err(error) = self.persist_config() {
            self.config.cloud_bindings = previous;
            return Err(error);
        }
        Ok(binding)
    }
}

#[cfg(test)]
#[path = "tests/cloud_bindings.rs"]
mod tests;
