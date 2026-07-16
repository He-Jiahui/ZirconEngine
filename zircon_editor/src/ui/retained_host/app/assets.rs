use std::sync::Arc;

use zircon_runtime::asset::AssetManager;
use zircon_runtime::core::framework::asset::ResourceManager;

use crate::ui::host::editor_asset_manager::EditorAssetManager as EditorAssetManagerContract;

use super::RetainedEditorHost;

mod bridge;
mod controls;
mod refresh;
mod workspace;

impl RetainedEditorHost {
    fn asset_manager_at_use_point(
        &self,
    ) -> Result<Arc<dyn AssetManager>, zircon_runtime::core::CoreError> {
        self.resource_manager_resolver
            .resolve(self.asset_manager.clone())
    }

    fn editor_asset_manager_at_use_point(
        &self,
    ) -> Result<Arc<dyn EditorAssetManagerContract>, zircon_runtime::core::CoreError> {
        self.resource_manager_resolver
            .resolve(self.editor_asset_manager.clone())
    }

    fn resolve_resource_manager(
        &self,
    ) -> Result<Arc<dyn ResourceManager>, zircon_runtime::core::CoreError> {
        self.resource_manager_resolver
            .resolve(self.resource_manager.clone())
    }
}
