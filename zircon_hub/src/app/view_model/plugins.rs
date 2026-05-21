use slint::SharedString;

use crate::plugins::{PluginCatalogEntry, ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::PluginData;
use super::localization;

pub(super) fn plugin_items(snapshot: &HubSnapshot) -> Vec<PluginData> {
    snapshot
        .plugins
        .iter()
        .map(|plugin| plugin_data(plugin, snapshot.settings.language))
        .collect()
}

fn plugin_data(plugin: &PluginCatalogEntry, language: HubLanguage) -> PluginData {
    PluginData {
        id: shared(plugin.id.clone()),
        title: shared(plugin.display_name.clone()),
        category: shared(plugin.category.clone()),
        scope: shared(plugin_scope_label(&plugin.scope, language)),
        maturity: shared(plugin.maturity.clone()),
        packaging: shared(packaging_label(&plugin.default_packaging, language)),
        modules: shared(plugin.module_count.to_string()),
        description: shared(if plugin.description.trim().is_empty() {
            localization::text(language, "No description", "无描述").to_string()
        } else {
            plugin.description.clone()
        }),
        path: shared(plugin.package_root.to_string_lossy().into_owned()),
    }
}

fn plugin_scope_label(scope: &str, language: HubLanguage) -> String {
    match scope {
        PROJECT_PLUGIN_SCOPE => {
            localization::text(language, "Selected Project", "选中项目").to_string()
        }
        ENGINE_PLUGIN_SCOPE => {
            localization::text(language, "Source Engine", "Source Engine").to_string()
        }
        _ => scope.to_string(),
    }
}

fn packaging_label(default_packaging: &[String], language: HubLanguage) -> String {
    if default_packaging.is_empty() {
        return localization::text(language, "manifest only", "仅清单").to_string();
    }
    default_packaging.join(", ")
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::plugins::PluginCatalogEntry;
    use crate::settings::HubSettings;
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn plugin_items_project_manifest_metadata() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Plugins,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: vec![PluginCatalogEntry {
                id: "demo".to_string(),
                display_name: "Demo Plugin".to_string(),
                description: String::new(),
                category: "runtime".to_string(),
                maturity: "beta".to_string(),
                default_packaging: vec!["native_dynamic".to_string()],
                module_count: 2,
                scope: crate::plugins::ENGINE_PLUGIN_SCOPE.to_string(),
                package_root: PathBuf::from("E:/plugins/demo"),
                manifest_path: PathBuf::from("E:/plugins/demo/plugin.toml"),
            }],
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let plugins = plugin_items(&snapshot);

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].title, SharedString::from("Demo Plugin"));
        assert_eq!(plugins[0].packaging, SharedString::from("native_dynamic"));
        assert_eq!(plugins[0].modules, SharedString::from("2"));
        assert_eq!(plugins[0].scope, SharedString::from("Source Engine"));
        assert_eq!(plugins[0].description, SharedString::from("No description"));
    }
}
