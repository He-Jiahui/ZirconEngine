use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{project_asset_manager_handle, ProjectAssetManager};
use zircon_runtime::asset::{asset_manager_handle, AssetManager};
use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime::core::manager::{
    resolve_manager_service, resource_manager_handle, ManagerServiceHandle,
};
use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};

use crate::ui::host::editor_asset_manager::{
    editor_asset_manager_handle, EditorAssetManager as EditorAssetManagerContract,
};

/// Retained-host access to the asset-related runtime services it owns.
///
/// Stable handles and a weak runtime reference stay in this leaf so UI callers cannot resolve
/// unrelated runtime services or retain the runtime lifetime.
pub(super) struct RetainedHostAssetRuntimeAccess {
    core: CoreWeak,
    asset_manager: ManagerServiceHandle<dyn AssetManager>,
    project_asset_manager: ManagerServiceHandle<ProjectAssetManager>,
    editor_asset_manager: ManagerServiceHandle<dyn EditorAssetManagerContract>,
    resource_manager: ManagerServiceHandle<dyn ResourceManager>,
}

impl RetainedHostAssetRuntimeAccess {
    pub(super) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        Ok(Self {
            core: core.downgrade(),
            asset_manager: asset_manager_handle(core)?,
            project_asset_manager: project_asset_manager_handle(core)?,
            editor_asset_manager: editor_asset_manager_handle(core)?,
            resource_manager: resource_manager_handle(core)?,
        })
    }

    pub(in crate::ui::retained_host::app) fn project_asset_manager(
        &self,
    ) -> Result<Arc<ProjectAssetManager>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, self.project_asset_manager.clone())
    }

    pub(in crate::ui::retained_host::app) fn asset_manager(
        &self,
    ) -> Result<Arc<dyn AssetManager>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, self.asset_manager.clone())
    }

    pub(in crate::ui::retained_host::app) fn editor_asset_manager(
        &self,
    ) -> Result<Arc<dyn EditorAssetManagerContract>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, self.editor_asset_manager.clone())
    }

    pub(in crate::ui::retained_host::app) fn resource_manager(
        &self,
    ) -> Result<Arc<dyn ResourceManager>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, self.resource_manager.clone())
    }

    fn core(&self) -> Result<CoreHandle, CoreError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable("CoreRuntime".to_owned()))
    }
}
