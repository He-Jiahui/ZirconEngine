use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetSelectionSnapshot, AssetSurfaceMode,
    AssetUtilityTab, AssetViewMode,
};

#[derive(Clone, Debug, Default)]
pub struct AssetWorkspaceSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub library_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub surface_mode: AssetSurfaceMode,
    pub view_mode: AssetViewMode,
    pub utility_tab: AssetUtilityTab,
    pub search_query: String,
    pub kind_filter: Option<ResourceKind>,
    pub folder_tree: Vec<AssetFolderSnapshot>,
    pub visible_folders: Vec<AssetFolderSnapshot>,
    pub visible_assets: Vec<AssetItemSnapshot>,
    pub selected_folder_id: Option<String>,
    pub selected_asset_uuid: Option<String>,
    pub selection: AssetSelectionSnapshot,
}
