use slint::SharedString;

use crate::learn::{LearnCatalogEntry, SELECTED_PROJECT_LEARN_SOURCE, SOURCE_ENGINE_LEARN_SOURCE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::LearnData;
use super::localization;

pub(super) fn learn_items(snapshot: &HubSnapshot) -> Vec<LearnData> {
    let mut resources = snapshot.learn_resources.iter().collect::<Vec<_>>();
    resources.sort_by(|left, right| {
        learn_source_priority(&left.source)
            .cmp(&learn_source_priority(&right.source))
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.category.cmp(&right.category))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.path.cmp(&right.path))
    });
    resources
        .into_iter()
        .enumerate()
        .map(|(index, resource)| learn_data(index, resource, snapshot.settings.language))
        .collect()
}

fn learn_data(index: usize, resource: &LearnCatalogEntry, language: HubLanguage) -> LearnData {
    LearnData {
        title: shared(resource.title.clone()),
        category: shared(resource.category.clone()),
        source: shared(learn_source_label(&resource.source, language)),
        summary: shared(if resource.summary.trim().is_empty() {
            localization::text(language, "Local documentation", "本地文档").to_string()
        } else {
            resource.summary.clone()
        }),
        path: shared(resource.path.to_string_lossy().into_owned()),
        accent: index as i32,
    }
}

fn learn_source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_LEARN_SOURCE => 0,
        SOURCE_ENGINE_LEARN_SOURCE => 1,
        _ => 2,
    }
}

fn learn_source_label(source: &str, language: HubLanguage) -> String {
    match source {
        SELECTED_PROJECT_LEARN_SOURCE => {
            localization::text(language, "Selected Project", "选中项目").to_string()
        }
        SOURCE_ENGINE_LEARN_SOURCE => {
            localization::text(language, "Source Engine", "Source Engine").to_string()
        }
        _ => source.to_string(),
    }
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::settings::HubSettings;
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn learn_items_project_local_docs() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Learn,
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
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: vec![LearnCatalogEntry {
                title: "Zircon Hub".to_string(),
                category: "Zircon hub".to_string(),
                source: SOURCE_ENGINE_LEARN_SOURCE.to_string(),
                summary: String::new(),
                path: PathBuf::from("E:/Zircon/docs/zircon_hub/index.md"),
            }],
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let items = learn_items(&snapshot);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, SharedString::from("Zircon Hub"));
        assert_eq!(items[0].source, SharedString::from("Source Engine"));
        assert_eq!(items[0].summary, SharedString::from("Local documentation"));
    }

    #[test]
    fn learn_items_orders_selected_project_docs_before_engine_docs() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Learn,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/Project")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: vec![
                LearnCatalogEntry {
                    title: "Engine Guide".to_string(),
                    category: "Engine".to_string(),
                    source: SOURCE_ENGINE_LEARN_SOURCE.to_string(),
                    summary: String::new(),
                    path: PathBuf::from("E:/Zircon/docs/engine/index.md"),
                },
                LearnCatalogEntry {
                    title: "Project Guide".to_string(),
                    category: "Guide".to_string(),
                    source: SELECTED_PROJECT_LEARN_SOURCE.to_string(),
                    summary: "Project-local onboarding.".to_string(),
                    path: PathBuf::from("E:/Project/docs/guide/project.md"),
                },
            ],
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let items = learn_items(&snapshot);

        assert_eq!(items[0].title, SharedString::from("Project Guide"));
        assert_eq!(items[0].source, SharedString::from("Selected Project"));
        assert_eq!(items[1].title, SharedString::from("Engine Guide"));
        assert_eq!(items[1].source, SharedString::from("Source Engine"));
    }
}
