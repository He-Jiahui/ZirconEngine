use std::sync::Arc;

use zircon_runtime::asset::{asset_manager_handle, AssetManager};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::{config_manager_handle, resolve_manager_service};
use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};
use zircon_runtime::runtime_diagnostics::collect_runtime_diagnostics;
use zircon_runtime::scene::{create_level, LevelMetadata, Scene};
use zircon_runtime::script::{
    HostHandle, PluginHostDriver, VmPluginManager, PLUGIN_HOST_DRIVER_NAME, VM_PLUGIN_MANAGER_NAME,
};

use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::extension::SaveReason;
use crate::ui::host::editor_asset_manager::{editor_asset_manager_handle, EditorAssetManager};
use crate::ui::workbench::view::ViewInstanceId;

use super::editor_document_autosave::ForegroundDocumentSaveJob;
use super::editor_error::EditorError;
use super::editor_subsystems::{
    editor_runtime_sandbox_enabled_from_config, editor_subsystem_report_from_config,
    EditorSubsystemReport, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
};
use super::module::EDITOR_MANAGER_NAME;
use super::EditorManager;

/// Host-private runtime access with only the operations the UI host actually owns.
///
/// `CoreWeak` stays contained here so workbench, project, plugin, and document owners cannot
/// resolve arbitrary runtime services through a generic locator.
#[derive(Clone)]
pub(super) struct EditorHostRuntimeServices {
    core: CoreWeak,
}

impl EditorHostRuntimeServices {
    pub(super) fn new(core: &CoreHandle) -> Self {
        Self {
            core: core.downgrade(),
        }
    }

    pub(super) fn subsystem_report(&self) -> Result<EditorSubsystemReport, EditorError> {
        Ok(editor_subsystem_report_from_config(
            self.core()?
                .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
                .ok(),
        ))
    }

    pub(super) fn runtime_sandbox_enabled(&self) -> Result<bool, EditorError> {
        Ok(editor_runtime_sandbox_enabled_from_config(
            self.core()?
                .load_config::<bool>(EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY)
                .ok(),
        ))
    }

    pub(super) fn capability_configuration(
        &self,
    ) -> Result<EditorCapabilityConfiguration, EditorError> {
        Ok(EditorCapabilityConfiguration { core: self.core()? })
    }

    pub(super) fn config_manager(&self) -> Result<Arc<dyn ConfigManager>, EditorError> {
        let core = self.core()?;
        Ok(resolve_manager_service(
            &core,
            config_manager_handle(&core)?,
        )?)
    }

    pub(super) fn asset_manager(&self) -> Result<Arc<dyn AssetManager>, EditorError> {
        let core = self.core()?;
        Ok(resolve_manager_service(
            &core,
            asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn editor_asset_manager(&self) -> Result<Arc<dyn EditorAssetManager>, EditorError> {
        let core = self.core()?;
        Ok(resolve_manager_service(
            &core,
            editor_asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn prepare_authoring_world(
        &self,
        scene: Scene,
    ) -> Result<AuthoringWorldSeed, EditorError> {
        Ok(AuthoringWorldSeed::from(create_level(
            &self.core()?,
            scene,
            LevelMetadata::default(),
        )?))
    }

    pub(super) fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        self.core()
            .map(|core| collect_runtime_diagnostics(&core))
            .unwrap_or_default()
    }

    pub(super) fn vm_host_capabilities(&self) -> Result<EditorVmHostCapabilityAccess, EditorError> {
        let core = self.core()?;
        let driver = core.resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)?;
        Ok(EditorVmHostCapabilityAccess { driver })
    }

    pub(super) fn vm_plugin_manager(&self) -> Result<Arc<VmPluginManager>, EditorError> {
        Ok(self
            .core()?
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)?)
    }

    pub(super) fn foreground_document_save_job(
        &self,
        instance_id: ViewInstanceId,
        reason: SaveReason,
    ) -> Result<ForegroundDocumentSaveJob, EditorError> {
        let manager = self
            .core()?
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        Ok(ForegroundDocumentSaveJob::new(
            Arc::downgrade(&manager),
            instance_id,
            reason,
        ))
    }

    fn core(&self) -> Result<CoreHandle, EditorError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::RuntimeUnavailable.into())
    }
}

/// A short-lived, typed config transaction that keeps the runtime alive through rollback.
pub(super) struct EditorCapabilityConfiguration {
    core: CoreHandle,
}

/// A short-lived VM host capability access that resolves the runtime driver once per bootstrap.
pub(super) struct EditorVmHostCapabilityAccess {
    driver: Arc<PluginHostDriver>,
}

impl EditorVmHostCapabilityAccess {
    pub(super) fn register_capability(
        &self,
        capability: String,
    ) -> Result<HostHandle, EditorError> {
        self.driver
            .registry()
            .register_capability(capability)
            .map_err(|error| EditorError::Registry(error.to_string()))
    }
}

impl EditorCapabilityConfiguration {
    pub(super) fn enabled_subsystems(&self) -> Vec<String> {
        self.core
            .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
            .unwrap_or_default()
    }

    pub(super) fn store_enabled_subsystems(&self, capabilities: &[String]) {
        self.core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(capabilities),
        );
    }

    pub(super) fn subsystem_report(&self) -> EditorSubsystemReport {
        editor_subsystem_report_from_config(
            self.core
                .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
                .ok(),
        )
    }
}
