use std::sync::Arc;

use crate::core::asset::AssetCreationMenuGeneration;
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetFolderSnapshot, AssetSelectionSnapshot, AssetSurfaceMode, AssetUtilityTab, AssetViewMode,
    AssetWorkspaceItemGeneration,
};

#[derive(Clone, Debug, Default)]
/// 单资产面的发布快照；绘制、虚拟化与pointer使用同一行代次和选择详情。
pub struct AssetWorkspaceSnapshot {
    pub project_name: String,
    pub project_root: String,
    pub assets_root: String,
    pub cache_root: String,
    pub default_scene_uri: String,
    /// workspace发布代次，包含目录、预览与资源更新；不能只当source catalog修订号。
    pub catalog_revision: u64,
    pub surface_mode: AssetSurfaceMode,
    pub view_mode: AssetViewMode,
    pub utility_tab: AssetUtilityTab,
    pub search_query: String,
    pub mesh_import_path: String,
    pub kind_filter: Option<ResourceKind>,
    pub folder_tree: Vec<AssetFolderSnapshot>,
    pub visible_folders: Vec<AssetFolderSnapshot>,
    /// 不可变共享行代次；行索引仅在该代次内有效，跨代次需按uuid/locator重新定位。
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
    /// 专供内容/引用pointer与拖动查询；项目元数据、目录行和creation menu在此投影中为空。
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
