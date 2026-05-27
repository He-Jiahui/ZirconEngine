use slint::SharedString;

use crate::plugins::{PluginCatalogEntry, ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::PluginData;
use super::localization;

pub(super) fn plugin_items(snapshot: &HubSnapshot) -> Vec<PluginData> {
    let mut plugins = snapshot.plugins.iter().collect::<Vec<_>>();
    plugins.sort_by(|left, right| {
        plugin_scope_priority(&left.scope)
            .cmp(&plugin_scope_priority(&right.scope))
            .then_with(|| left.category.cmp(&right.category))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.package_root.cmp(&right.package_root))
    });
    plugins
        .into_iter()
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

fn plugin_scope_priority(scope: &str) -> u8 {
    match scope {
        PROJECT_PLUGIN_SCOPE => 0,
        ENGINE_PLUGIN_SCOPE => 1,
        _ => 2,
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

    use crate::plugins::{PluginCatalogEntry, ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE};
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

    #[test]
    fn plugin_items_orders_project_manifests_before_engine_plugins() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Plugins,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/projects/demo")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: vec![
                PluginCatalogEntry {
                    id: "engine_runtime".to_string(),
                    display_name: "Engine Runtime".to_string(),
                    description: "Engine plugin".to_string(),
                    category: "runtime".to_string(),
                    maturity: "stable".to_string(),
                    default_packaging: vec!["native_dynamic".to_string()],
                    module_count: 1,
                    scope: ENGINE_PLUGIN_SCOPE.to_string(),
                    package_root: PathBuf::from("E:/engine/zircon_plugins/engine_runtime"),
                    manifest_path: PathBuf::from(
                        "E:/engine/zircon_plugins/engine_runtime/plugin.toml",
                    ),
                },
                PluginCatalogEntry {
                    id: "project_runtime".to_string(),
                    display_name: "Project Runtime".to_string(),
                    description: "Project plugin".to_string(),
                    category: "runtime".to_string(),
                    maturity: "beta".to_string(),
                    default_packaging: Vec::new(),
                    module_count: 2,
                    scope: PROJECT_PLUGIN_SCOPE.to_string(),
                    package_root: PathBuf::from("E:/projects/demo/Plugins/project_runtime"),
                    manifest_path: PathBuf::from(
                        "E:/projects/demo/Plugins/project_runtime/plugin.toml",
                    ),
                },
            ],
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let plugins = plugin_items(&snapshot);

        assert_eq!(plugins[0].id, SharedString::from("project_runtime"));
        assert_eq!(plugins[0].scope, SharedString::from("Selected Project"));
        assert_eq!(plugins[0].packaging, SharedString::from("manifest only"));
        assert_eq!(plugins[1].id, SharedString::from("engine_runtime"));
        assert_eq!(plugins[1].scope, SharedString::from("Source Engine"));
    }
}
