use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::super::super::data::{
    AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData,
};
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_activity_asset_tree_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_activity_asset_content_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_activity_asset_content_items(&self, _value: ModelRc<AssetItemData>) {}
    pub(crate) fn set_activity_asset_selection(&self, _value: AssetSelectionData) {}
    pub(crate) fn set_activity_asset_references(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_activity_asset_used_by(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_activity_asset_search_query(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_kind_filter(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_view_mode(&self, _value: SharedString) {}
    pub(crate) fn set_activity_asset_utility_tab(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_tree_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_browser_asset_content_folders(&self, _value: ModelRc<AssetFolderData>) {}
    pub(crate) fn set_browser_asset_content_items(&self, _value: ModelRc<AssetItemData>) {}
    pub(crate) fn set_browser_asset_selection(&self, _value: AssetSelectionData) {}
    pub(crate) fn set_browser_asset_references(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_browser_asset_used_by(&self, _value: ModelRc<AssetReferenceData>) {}
    pub(crate) fn set_browser_asset_search_query(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_kind_filter(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_view_mode(&self, _value: SharedString) {}
    pub(crate) fn set_browser_asset_utility_tab(&self, _value: SharedString) {}
}
