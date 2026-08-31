use std::sync::Arc;

use crate::core::asset::AssetCreationMenuGeneration;
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetFolderSnapshot, AssetSelectionSnapshot, AssetSurfaceMode, AssetUtilityTab, AssetViewMode,
    AssetWorkspaceItemGeneration,
};

#[derive(Clone, Debug, Default)]
pub struct AssetWorkspaceSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub cache_root: String,
    pub default_scene_uri: String,
    pub catalog_revision: u64,
    pub surface_mode: AssetSurfaceMode,
    pub view_mode: AssetViewMode,
    pub utility_tab: AssetUtilityTab,
    pub search_query: String,
    pub mesh_import_path: String,
    pub kind_filter: Option<ResourceKind>,
    pub folder_tree: Vec<AssetFolderSnapshot>,
    pub visible_folders: Vec<AssetFolderSnapshot>,
    pub visible_assets: AssetWorkspaceItemGeneration,
    pub creation_menu: Arc<AssetCreationMenuGeneration>,
    pub selected_folder_id: Option<String>,
    pub selected_asset_uuid: Option<String>,
    pub selection: AssetSelectionSnapshot,
}

impl AssetWorkspaceSnapshot {
    /// Retained pointer surfaces only need the published asset rows and selected-detail data.
    /// Keep this projection separate from the full pane snapshot so pointer publication does not
    /// clone unrelated project metadata, folder rows, or creation-menu payloads.
    pub(crate) fn pointer_projection(&self) -> Self {
        Self {
            catalog_revision: self.catalog_revision,
            surface_mode: self.surface_mode,
            view_mode: self.view_mode,
            utility_tab: self.utility_tab,
            visible_assets: self.visible_assets.clone(),
            selection: self.selection.clone(),
            ..Self::default()
        }
    }
}
