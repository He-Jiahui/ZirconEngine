//! EditorState保留UI选择，资产管理器提供不可变catalog/details/resource代次；同步不能代替资源加载。
use crate::core::asset::AssetTypeId;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::resource::ResourceManagementGeneration;
use zircon_runtime_interface::resource::ResourceKind;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetDetailsGeneration,
};
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode};
use crate::ui::workbench::state::EditorState;

impl EditorState {
    pub fn sync_asset_catalog(&mut self, catalog: Arc<EditorAssetCatalogGeneration>) {
        self.asset_workspace.sync_catalog(catalog);
    }

    /// 局部目录同步：差异UUID须覆盖该发布代次的全部影响，不能把未知全量变化伪装成空差异。
    pub fn sync_asset_catalog_changes(
        &mut self,
        catalog: Arc<EditorAssetCatalogGeneration>,
        changed_asset_uuids: &[String],
    ) {
        self.asset_workspace
            .sync_catalog_changes(catalog, changed_asset_uuids);
    }

    pub fn sync_asset_details(&mut self, details: Option<Arc<EditorAssetDetailsGeneration>>) {
        self.asset_workspace.sync_selected_details(details);
    }

    pub fn sync_asset_resources(&mut self, resources: Arc<ResourceManagementGeneration>) -> bool {
        self.asset_workspace.sync_resources(resources)
    }

    /// 用受影响locator同步资源代次；返回值表示呈现变化，实际加载结果来自资源generation。
    pub fn sync_asset_resource_changes(
        &mut self,
        resources: Arc<ResourceManagementGeneration>,
        changed_locators: &[String],
    ) -> bool {
        self.asset_workspace
            .sync_resource_changes(resources, changed_locators)
    }

    pub fn select_asset_folder(&mut self, folder_id: impl Into<String>) -> bool {
        self.asset_workspace.select_folder(folder_id)
    }

    pub fn select_asset(&mut self, asset_uuid: Option<String>) -> bool {
        self.asset_workspace.select_asset(asset_uuid)
    }

    pub(crate) fn selected_asset_uuid(&self) -> Option<&str> {
        self.asset_workspace.selected_asset_uuid()
    }

    pub fn navigate_to_asset(&mut self, asset_uuid: &str) -> bool {
        self.asset_workspace.navigate_to_asset(asset_uuid)
    }

    pub(crate) fn asset_type_id_for_locator(&self, locator: &AssetUri) -> Option<AssetTypeId> {
        self.asset_workspace.asset_type_id_for_locator(locator)
    }

    pub fn set_asset_search_query(&mut self, query: impl Into<String>) -> bool {
        self.asset_workspace.set_search_query(query)
    }

    pub fn set_asset_kind_filter(&mut self, kind_filter: Option<ResourceKind>) -> bool {
        self.asset_workspace.set_kind_filter(kind_filter)
    }

    pub fn set_asset_activity_view_mode(&mut self, view_mode: AssetViewMode) -> bool {
        self.asset_workspace.set_activity_view_mode(view_mode)
    }

    pub fn set_asset_browser_view_mode(&mut self, view_mode: AssetViewMode) -> bool {
        self.asset_workspace.set_browser_view_mode(view_mode)
    }

    pub fn set_asset_activity_tab(&mut self, tab: AssetUtilityTab) -> bool {
        self.asset_workspace.set_activity_utility_tab(tab)
    }

    pub fn set_asset_browser_tab(&mut self, tab: AssetUtilityTab) -> bool {
        self.asset_workspace.set_browser_utility_tab(tab)
    }
}
use std::sync::Arc;
