use zircon_runtime_interface::resource::{ResourceKind, ResourceRecord};

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
};
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode};
use crate::ui::workbench::state::EditorState;

impl EditorState {
    pub fn sync_asset_catalog(&mut self, catalog: EditorAssetCatalogSnapshotRecord) {
        self.asset_workspace.sync_catalog(catalog);
    }

    pub fn sync_asset_details(&mut self, details: Option<EditorAssetDetailsRecord>) {
        self.asset_workspace.sync_selected_details(details);
    }

    pub fn sync_asset_resources(&mut self, resources: Vec<ResourceRecord>) {
        self.asset_workspace.sync_resources(resources);
    }

    pub fn select_asset_folder(&mut self, folder_id: impl Into<String>) {
        self.asset_workspace.select_folder(folder_id);
    }

    pub fn select_asset(&mut self, asset_uuid: Option<String>) {
        self.asset_workspace.select_asset(asset_uuid);
    }

    pub fn navigate_to_asset(&mut self, asset_uuid: &str) {
        self.asset_workspace.navigate_to_asset(asset_uuid);
    }

    pub fn set_asset_search_query(&mut self, query: impl Into<String>) {
        self.asset_workspace.set_search_query(query);
    }

    pub fn set_asset_kind_filter(&mut self, kind_filter: Option<ResourceKind>) {
        self.asset_workspace.set_kind_filter(kind_filter);
    }

    pub fn set_asset_activity_view_mode(&mut self, view_mode: AssetViewMode) {
        self.asset_workspace.set_activity_view_mode(view_mode);
    }

    pub fn set_asset_browser_view_mode(&mut self, view_mode: AssetViewMode) {
        self.asset_workspace.set_browser_view_mode(view_mode);
    }

    pub fn set_asset_activity_tab(&mut self, tab: AssetUtilityTab) {
        self.asset_workspace.set_activity_utility_tab(tab);
    }

    pub fn set_asset_browser_tab(&mut self, tab: AssetUtilityTab) {
        self.asset_workspace.set_browser_utility_tab(tab);
    }
}
