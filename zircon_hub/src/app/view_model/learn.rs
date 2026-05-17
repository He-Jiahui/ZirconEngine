use slint::SharedString;

use crate::learn::LearnCatalogEntry;
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::LearnData;
use super::localization;

pub(super) fn learn_items(snapshot: &HubSnapshot) -> Vec<LearnData> {
    snapshot
        .learn_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| learn_data(index, resource, snapshot.settings.language))
        .collect()
}

fn learn_data(index: usize, resource: &LearnCatalogEntry, language: HubLanguage) -> LearnData {
    LearnData {
        title: shared(resource.title.clone()),
        category: shared(resource.category.clone()),
        summary: shared(if resource.summary.trim().is_empty() {
            localization::text(language, "Local documentation", "本地文档").to_string()
        } else {
            resource.summary.clone()
        }),
        path: shared(resource.path.to_string_lossy().into_owned()),
        accent: index as i32,
    }
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::settings::HubSettings;
    use crate::state::{HubPage, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus};

    use super::*;

    #[test]
    fn learn_items_project_local_docs() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Learn,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            assets: Vec::new(),
            learn_resources: vec![LearnCatalogEntry {
                title: "Zircon Hub".to_string(),
                category: "Zircon hub".to_string(),
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
        assert_eq!(items[0].summary, SharedString::from("Local documentation"));
    }
}
