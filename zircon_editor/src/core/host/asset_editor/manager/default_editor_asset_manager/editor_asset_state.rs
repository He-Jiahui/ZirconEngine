use std::collections::HashMap;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{AssetUri, AssetUuid};

use crate::{AssetCatalogRecord, PreviewCache, PreviewScheduler, ReferenceGraph};

#[derive(Clone, Debug, Default)]
pub(in crate::core::host::asset_editor::manager) struct EditorAssetState {
    pub(in crate::core::host::asset_editor::manager) project_root: Option<PathBuf>,
    pub(in crate::core::host::asset_editor::manager) assets_root: Option<PathBuf>,
    pub(in crate::core::host::asset_editor::manager) library_root: Option<PathBuf>,
    pub(in crate::core::host::asset_editor::manager) project_name: String,
    pub(in crate::core::host::asset_editor::manager) default_scene_uri: Option<AssetUri>,
    pub(in crate::core::host::asset_editor::manager) catalog_revision: u64,
    pub(in crate::core::host::asset_editor::manager) project: Option<ProjectManager>,
    pub(in crate::core::host::asset_editor::manager) catalog_by_uuid:
        HashMap<AssetUuid, AssetCatalogRecord>,
    pub(in crate::core::host::asset_editor::manager) uuid_by_locator: HashMap<AssetUri, AssetUuid>,
    pub(in crate::core::host::asset_editor::manager) reference_graph: ReferenceGraph,
    pub(in crate::core::host::asset_editor::manager) preview_cache: Option<PreviewCache>,
    pub(in crate::core::host::asset_editor::manager) preview_scheduler: PreviewScheduler,
}
