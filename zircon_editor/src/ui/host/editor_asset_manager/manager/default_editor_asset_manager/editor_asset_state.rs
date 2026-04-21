use std::collections::HashMap;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetUri, AssetUuid};

use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, PreviewCache, PreviewScheduler, ReferenceGraph,
};

#[derive(Clone, Debug, Default)]
pub(in crate::ui::host::editor_asset_manager::manager) struct EditorAssetState {
    pub(in crate::ui::host::editor_asset_manager::manager) project_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) assets_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) library_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) project_name: String,
    pub(in crate::ui::host::editor_asset_manager::manager) default_scene_uri: Option<AssetUri>,
    pub(in crate::ui::host::editor_asset_manager::manager) catalog_revision: u64,
    pub(in crate::ui::host::editor_asset_manager::manager) project: Option<ProjectManager>,
    pub(in crate::ui::host::editor_asset_manager::manager) catalog_by_uuid:
        HashMap<AssetUuid, AssetCatalogRecord>,
    pub(in crate::ui::host::editor_asset_manager::manager) uuid_by_locator:
        HashMap<AssetUri, AssetUuid>,
    pub(in crate::ui::host::editor_asset_manager::manager) reference_graph: ReferenceGraph,
    pub(in crate::ui::host::editor_asset_manager::manager) preview_cache: Option<PreviewCache>,
    pub(in crate::ui::host::editor_asset_manager::manager) preview_scheduler: PreviewScheduler,
}
