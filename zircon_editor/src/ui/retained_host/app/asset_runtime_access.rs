use std::sync::Arc;

use zircon_runtime::asset::{asset_manager_handle, AssetManager};
use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime::core::manager::{ManagerResolver, ManagerServiceHandle};
use zircon_runtime::core::{CoreError, CoreHandle};

use crate::ui::host::editor_asset_manager::{
    editor_asset_manager_handle, EditorAssetManager as EditorAssetManagerContract,
};

/// Retained-host access to the three asset-related runtime services it owns.
///
/// The generic resolver and stable handles stay in this leaf so UI callers cannot resolve
/// unrelated runtime services.
pub(super) struct RetainedHostAssetRuntimeAccess {
    resolver: ManagerResolver,
    asset_manager: ManagerServiceHandle<dyn AssetManager>,
    editor_asset_manager: ManagerServiceHandle<dyn EditorAssetManagerContract>,
    resource_manager: ManagerServiceHandle<dyn ResourceManager>,
}

impl RetainedHostAssetRuntimeAccess {
    pub(super) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let resolver = ManagerResolver::new(core.clone());
        Ok(Self {
            asset_manager: asset_manager_handle(core)?,
            editor_asset_manager: editor_asset_manager_handle(core)?,
            resource_manager: resolver.resource_handle()?,
            resolver,
        })
    }

    pub(in crate::ui::retained_host::app) fn asset_manager(
        &self,
    ) -> Result<Arc<dyn AssetManager>, CoreError> {
        self.resolver.resolve(self.asset_manager.clone())
    }

    pub(in crate::ui::retained_host::app) fn editor_asset_manager(
        &self,
    ) -> Result<Arc<dyn EditorAssetManagerContract>, CoreError> {
        self.resolver.resolve(self.editor_asset_manager.clone())
    }

    pub(in crate::ui::retained_host::app) fn resource_manager(
        &self,
    ) -> Result<Arc<dyn ResourceManager>, CoreError> {
        self.resolver.resolve(self.resource_manager.clone())
    }
}
