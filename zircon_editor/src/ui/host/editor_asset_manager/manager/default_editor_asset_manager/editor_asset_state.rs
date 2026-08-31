use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::AssetUri;

use crate::core::asset::EditorAssetIndex;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, PreviewCache, PreviewScheduler,
};

use super::DefaultEditorAssetManager;

pub(in crate::ui::host::editor_asset_manager::manager) fn read_editor_asset_state_recovering_poison(
    state: &RwLock<EditorAssetState>,
) -> RwLockReadGuard<'_, EditorAssetState> {
    state
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(in crate::ui::host::editor_asset_manager::manager) fn write_editor_asset_state_recovering_poison(
    state: &RwLock<EditorAssetState>,
) -> RwLockWriteGuard<'_, EditorAssetState> {
    state
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(in crate::ui::host::editor_asset_manager::manager) fn lock_editor_asset_gate_recovering_poison(
    gate: &Mutex<()>,
) -> MutexGuard<'_, ()> {
    gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Debug, Default)]
pub(in crate::ui::host::editor_asset_manager::manager) struct EditorAssetState {
    pub(in crate::ui::host::editor_asset_manager::manager) project_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) assets_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) cache_root: Option<PathBuf>,
    pub(in crate::ui::host::editor_asset_manager::manager) project_name: String,
    pub(in crate::ui::host::editor_asset_manager::manager) default_scene_uri: Option<AssetUri>,
    pub(in crate::ui::host::editor_asset_manager::manager) catalog_generation:
        Arc<EditorAssetCatalogGeneration>,
    pub(in crate::ui::host::editor_asset_manager::manager) project: Option<ProjectManager>,
    pub(in crate::ui::host::editor_asset_manager::manager) asset_index:
        Option<Arc<Mutex<EditorAssetIndex>>>,
    pub(in crate::ui::host::editor_asset_manager::manager) preview_cache: Option<PreviewCache>,
    pub(in crate::ui::host::editor_asset_manager::manager) preview_scheduler: PreviewScheduler,
}

impl DefaultEditorAssetManager {
    pub(in crate::ui::host::editor_asset_manager::manager) fn advance_source_sync_epoch(
        &self,
    ) -> u64 {
        let previous = self
            .source_sync_epoch
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |epoch| {
                epoch.checked_add(1)
            })
            .expect("editor asset source-sync epoch exhausted");
        previous
            .checked_add(1)
            .expect("a successfully advanced source-sync epoch must be representable")
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn read_state_recovering_poison(
        &self,
    ) -> RwLockReadGuard<'_, EditorAssetState> {
        read_editor_asset_state_recovering_poison(self.state.as_ref())
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn write_state_recovering_poison(
        &self,
    ) -> RwLockWriteGuard<'_, EditorAssetState> {
        write_editor_asset_state_recovering_poison(self.state.as_ref())
    }
}
