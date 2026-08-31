use std::sync::Arc;

use zircon_runtime::asset::AssetManager;
use zircon_runtime::core::framework::asset::ResourceManager;

use crate::ui::host::editor_asset_manager::EditorAssetManager as EditorAssetManagerContract;

use super::RetainedEditorHost;

mod bridge;
mod controls;
mod deletion;
mod refresh;
mod relocation;
mod workspace;

pub(super) use deletion::PendingAssetDeletion;
pub(super) use refresh::{AssetRefreshAccumulator, AssetRefreshQueueAgeState};
pub(super) use relocation::PendingAssetRelocation;
pub(super) use workspace::{
    ActiveSceneReloadAdmissionState, ActiveSceneReloadConflict, PendingActiveSceneReload,
    PendingModelImport,
};

impl RetainedEditorHost {
    fn asset_manager_at_use_point(
        &self,
    ) -> Result<Arc<dyn AssetManager>, zircon_runtime::core::CoreError> {
        self.asset_runtime_access.asset_manager()
    }

    pub(in crate::ui::retained_host::app) fn editor_asset_manager_at_use_point(
        &self,
    ) -> Result<Arc<dyn EditorAssetManagerContract>, zircon_runtime::core::CoreError> {
        self.asset_runtime_access.editor_asset_manager()
    }

    fn resolve_resource_manager(
        &self,
    ) -> Result<Arc<dyn ResourceManager>, zircon_runtime::core::CoreError> {
        self.asset_runtime_access.resource_manager()
    }
}
