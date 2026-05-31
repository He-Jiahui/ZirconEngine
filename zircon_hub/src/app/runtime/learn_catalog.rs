use std::path::PathBuf;

use crate::error::HubError;
use crate::learn::discover_learn_catalog_for_scope;
use crate::process::{open_folder, OpenFolderCommand};
use crate::projects::project_filesystem_path_key;
use crate::state::TaskStatus;

use super::HubRuntime;

impl HubRuntime {
    pub(super) fn refresh_learn_catalog(&mut self) -> Result<(), HubError> {
        self.learn_catalog = discover_learn_catalog_for_scope(
            self.selected_project_catalog_root(),
            self.source_engine_catalog_roots(),
        )?;
        Ok(())
    }

    pub(super) fn open_learn_resource(&mut self, resource_path: &str) -> Result<(), HubError> {
        let path = self.learn_resource_path_for_open(resource_path)?;
        let command = OpenFolderCommand::new(path.clone());
        open_folder(&command)?;
        self.task_status =
            TaskStatus::success("Resource opened", path.to_string_lossy().into_owned());
        Ok(())
    }

    #[cfg(test)]
    fn open_learn_resource_command(
        &self,
        resource_path: &str,
    ) -> Result<OpenFolderCommand, HubError> {
        self.learn_resource_path_for_open(resource_path)
            .map(OpenFolderCommand::new)
    }

    fn learn_resource_path_for_open(&self, resource_path: &str) -> Result<PathBuf, HubError> {
        let trimmed = resource_path.trim();
        if trimmed.is_empty() {
            return Err(HubError::message("Learn resource path is empty"));
        }
        let requested_path = PathBuf::from(trimmed);
        let requested_key = project_filesystem_path_key(&requested_path);
        let Some(catalog_path) = self
            .learn_catalog
            .iter()
            .map(|resource| &resource.path)
            .find(|path| project_filesystem_path_key(path) == requested_key)
            .cloned()
        else {
            return Err(HubError::message(format!(
                "Learn resource is not in the current catalog: {trimmed}"
            )));
        };
        if !catalog_path.is_file() {
            return Err(HubError::message(format!(
                "Learn resource is no longer available: {}",
                catalog_path.to_string_lossy()
            )));
        }
        Ok(catalog_path)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::app::runtime::HubRuntime;
    use crate::learn::{LearnCatalogEntry, SELECTED_PROJECT_LEARN_SOURCE};
    use crate::projects::{now_unix_ms, ProjectTemplate, RecentProject};
    use crate::settings::HubConfig;
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };
    use crate::team::TeamOverview;

    #[test]
    fn learn_resource_open_requires_catalog_entry_with_existing_file() {
        let temp = temp_test_dir("zircon-hub-learn-open-resource");
        let docs_root = temp.join("docs").join("guide");
        fs::create_dir_all(&docs_root).expect("docs root should be created");
        let doc_path = docs_root.join("intro.md");
        fs::write(&doc_path, "# Intro\n\nProject docs.").expect("doc should be written");
        let mut runtime = runtime_with_projects(vec![RecentProject::new("Demo", &temp, 10)]);
        runtime.learn_catalog = vec![LearnCatalogEntry {
            title: "Intro".to_string(),
            category: "Guide".to_string(),
            source: SELECTED_PROJECT_LEARN_SOURCE.to_string(),
            summary: "Project docs.".to_string(),
            path: doc_path.clone(),
        }];

        assert_eq!(
            runtime
                .learn_resource_path_for_open(" ")
                .unwrap_err()
                .to_string(),
            "Learn resource path is empty"
        );
        assert!(runtime
            .learn_resource_path_for_open(
                temp.join("docs")
                    .join("missing.md")
                    .to_string_lossy()
                    .as_ref()
            )
            .unwrap_err()
            .to_string()
            .contains("not in the current catalog"));
        fs::remove_file(&doc_path).expect("doc should be removable");
        assert!(runtime
            .learn_resource_path_for_open(doc_path.to_string_lossy().as_ref())
            .unwrap_err()
            .to_string()
            .contains("no longer available"));
        fs::write(&doc_path, "# Intro\n").expect("doc should be restored");

        let resolved = runtime
            .learn_resource_path_for_open(doc_path.to_string_lossy().as_ref())
            .expect("cataloged existing doc should be openable");
        let command = runtime
            .open_learn_resource_command(doc_path.to_string_lossy().as_ref())
            .expect("cataloged existing doc should produce an open command");

        assert_eq!(resolved, doc_path);
        assert_eq!(command.args, vec![doc_path.to_string_lossy().into_owned()]);
        fs::remove_dir_all(temp).expect("temp dir should be removed");
    }

    fn runtime_with_projects(projects: Vec<RecentProject>) -> HubRuntime {
        HubRuntime {
            config_path: PathBuf::from("hub.toml"),
            editor_config_path: PathBuf::from("editor.json"),
            config: HubConfig {
                recent_projects: projects,
                ..HubConfig::default()
            },
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: ProjectTemplate::RenderableEmpty.id().to_string(),
            new_project_location: HubConfig::default().settings.default_project_dir,
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        }
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("{prefix}-{}-{}", std::process::id(), now_unix_ms()));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }
}
