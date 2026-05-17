use slint::SharedString;

use crate::assets::AssetCatalogEntry;
use crate::state::HubSnapshot;

use super::super::AssetData;

pub(super) fn asset_items(snapshot: &HubSnapshot) -> Vec<AssetData> {
    snapshot
        .assets
        .iter()
        .enumerate()
        .map(|(index, asset)| asset_data(index, asset))
        .collect()
}

fn asset_data(index: usize, asset: &AssetCatalogEntry) -> AssetData {
    AssetData {
        name: shared(asset.name.clone()),
        kind: shared(asset.kind.clone()),
        source: shared(asset.source.clone()),
        size: shared(format_size(asset.size_bytes)),
        path: shared(asset.path.to_string_lossy().into_owned()),
        accent: index as i32,
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
    use crate::state::{HubPage, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus};

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
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            assets: vec![AssetCatalogEntry {
                name: "diffuse.png".to_string(),
                kind: "image".to_string(),
                source: "Project".to_string(),
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
        assert_eq!(items[0].size, SharedString::from("1.5 KB"));
    }
}
