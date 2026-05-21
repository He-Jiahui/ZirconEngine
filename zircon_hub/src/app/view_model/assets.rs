use slint::SharedString;

use crate::assets::{AssetCatalogEntry, PROJECT_ASSET_SOURCE, SELECTED_PROJECT_ASSET_SOURCE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::AssetData;
use super::localization;

pub(super) fn asset_items(snapshot: &HubSnapshot) -> Vec<AssetData> {
    let mut assets = snapshot.assets.iter().collect::<Vec<_>>();
    assets.sort_by(|left, right| {
        asset_source_priority(&left.source)
            .cmp(&asset_source_priority(&right.source))
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.path.cmp(&right.path))
    });
    assets
        .into_iter()
        .enumerate()
        .map(|(index, asset)| asset_data(index, asset, snapshot.settings.language))
        .collect()
}

fn asset_data(index: usize, asset: &AssetCatalogEntry, language: HubLanguage) -> AssetData {
    AssetData {
        name: shared(asset.name.clone()),
        kind: shared(asset.kind.clone()),
        source: shared(asset_source_label(&asset.source, language)),
        size: shared(format_size(asset.size_bytes)),
        path: shared(asset.path.to_string_lossy().into_owned()),
        accent: index as i32,
    }
}

fn asset_source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_ASSET_SOURCE => 0,
        PROJECT_ASSET_SOURCE => 1,
        _ => 2,
    }
}

fn asset_source_label(source: &str, language: HubLanguage) -> String {
    match source {
        SELECTED_PROJECT_ASSET_SOURCE => {
            localization::text(language, "Selected Project", "选中项目").to_string()
        }
        PROJECT_ASSET_SOURCE => {
            localization::text(language, "Recent Project", "最近项目").to_string()
        }
        _ => match language {
            HubLanguage::English => format!("Source Engine / {source}"),
            HubLanguage::Chinese => format!("Source Engine / {source}"),
        },
    }
}

fn format_size(size_bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    let size = size_bytes as f64;
    if size_bytes < 1024 {
        format!("{size_bytes} B")
    } else if size < MIB {
        format!("{:.1} KB", size / KIB)
    } else {
        format!("{:.1} MB", size / MIB)
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
    fn format_size_uses_binary_units() {
        assert_eq!(format_size(12), "12 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.0 MB");
    }

    #[test]
    fn asset_items_project_catalog_entries() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Assets,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/Project")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: vec![AssetCatalogEntry {
                name: "diffuse.png".to_string(),
                kind: "image".to_string(),
                source: SELECTED_PROJECT_ASSET_SOURCE.to_string(),
                size_bytes: 1536,
                path: PathBuf::from("E:/Project/Assets/diffuse.png"),
            }],
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let items = asset_items(&snapshot);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, SharedString::from("diffuse.png"));
        assert_eq!(items[0].kind, SharedString::from("image"));
        assert_eq!(items[0].source, SharedString::from("Selected Project"));
        assert_eq!(items[0].size, SharedString::from("1.5 KB"));
    }

    #[test]
    fn asset_items_orders_selected_project_assets_before_engine_assets() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Assets,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/Project")),
            selected_template_id: "renderable-empty".to_string(),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: vec![
                AssetCatalogEntry {
                    name: "runtime.svg".to_string(),
                    kind: "image".to_string(),
                    source: "Runtime".to_string(),
                    size_bytes: 128,
                    path: PathBuf::from("E:/Repo/zircon_runtime/assets/runtime.svg"),
                },
                AssetCatalogEntry {
                    name: "hero.glb".to_string(),
                    kind: "model".to_string(),
                    source: SELECTED_PROJECT_ASSET_SOURCE.to_string(),
                    size_bytes: 4096,
                    path: PathBuf::from("E:/Project/Assets/hero.glb"),
                },
                AssetCatalogEntry {
                    name: "ambient.ogg".to_string(),
                    kind: "audio".to_string(),
                    source: PROJECT_ASSET_SOURCE.to_string(),
                    size_bytes: 2048,
                    path: PathBuf::from("E:/Other/assets/ambient.ogg"),
                },
            ],
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
        };

        let items = asset_items(&snapshot);

        assert_eq!(items[0].name, SharedString::from("hero.glb"));
        assert_eq!(items[0].source, SharedString::from("Selected Project"));
        assert_eq!(items[1].source, SharedString::from("Recent Project"));
        assert_eq!(
            items[2].source,
            SharedString::from("Source Engine / Runtime")
        );
    }
}
