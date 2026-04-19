use std::collections::HashMap;
use std::path::PathBuf;

use crate::project::ProjectManager;
use crate::{
    AssetCatalogRecord, AssetUri, AssetUuid, PreviewCache, PreviewScheduler, ReferenceGraph,
};

#[derive(Clone, Debug, Default)]
pub(in crate::editor::manager) struct EditorAssetState {
    pub(in crate::editor::manager) project_root: Option<PathBuf>,
    pub(in crate::editor::manager) assets_root: Option<PathBuf>,
    pub(in crate::editor::manager) library_root: Option<PathBuf>,
    pub(in crate::editor::manager) project_name: String,
    pub(in crate::editor::manager) default_scene_uri: Option<AssetUri>,
    pub(in crate::editor::manager) catalog_revision: u64,
    pub(in crate::editor::manager) project: Option<ProjectManager>,
    pub(in crate::editor::manager) catalog_by_uuid: HashMap<AssetUuid, AssetCatalogRecord>,
    pub(in crate::editor::manager) uuid_by_locator: HashMap<AssetUri, AssetUuid>,
    pub(in crate::editor::manager) reference_graph: ReferenceGraph,
    pub(in crate::editor::manager) preview_cache: Option<PreviewCache>,
    pub(in crate::editor::manager) preview_scheduler: PreviewScheduler,
}
