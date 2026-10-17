use std::sync::{Arc, Mutex};

use zircon_runtime::core::runtime::{ServiceCallGuard, ServiceHandle};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{asset_manager_handle, project_asset_manager_handle, AssetManager};
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::{
    config_manager_handle, resolve_manager_service, ManagerServiceHandle, CONFIG_MANAGER_NAME,
};
use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};
use zircon_runtime::scene::{LevelMetadata, Scene};
use zircon_runtime::script::{
    HostHandle, PluginHostDriver, VmPluginManager, PLUGIN_HOST_DRIVER_NAME, VM_PLUGIN_MANAGER_NAME,
};

use crate::core::asset::{EditorAssetImportFlow, EditorAssetIndex};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::extension::SaveReason;
use crate::core::jobs::EditorJobSystem;
use crate::core::logging::EditorLogService;
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
        let core = self.core()?;
        let config = resolve_manager_service(&core, config_manager_handle(&core)?)?;
        Ok(EditorCapabilityConfiguration {
            _core: core,
            config,
        })
    }

    pub(super) fn config_manager(
        &self,
    ) -> Result<ServiceCallGuard<dyn ConfigManager>, EditorError> {
        let core = self.core()?;
        Ok(resolve_manager_service(
            &core,
            config_manager_handle(&core)?,
        )?)
    }

    pub(super) fn asset_manager(&self) -> Result<ServiceCallGuard<dyn AssetManager>, EditorError> {
        let core = self.core()?;
        Ok(resolve_manager_service(
            &core,
            asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn editor_asset_manager(
        &self,
    ) -> Result<ServiceCallGuard<dyn EditorAssetManager>, EditorError> {
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
        Ok(AuthoringWorldSeed::from(
            zircon_runtime::scene::prepare_level(&self.core()?, scene, LevelMetadata::default())?,
        ))
    }

    pub(super) fn runtime_diagnostics(&self) -> RuntimeDiagnosticsSnapshot {
        self.core()
            .map(|core| core.runtime_diagnostics_snapshot())
            .unwrap_or_default()
    }

    pub(super) fn vm_host_capabilities(&self) -> Result<EditorVmHostCapabilityAccess, EditorError> {
        let core = self.core()?;
        let driver = core.resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)?;
        Ok(EditorVmHostCapabilityAccess { driver })
    }

    pub(super) fn vm_plugin_manager(
        &self,
    ) -> Result<ServiceCallGuard<VmPluginManager>, EditorError> {
        Ok(self
            .core()?
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)?)
    }

    pub(super) fn foreground_document_save_job(
        &self,
        original: ServiceHandle<EditorManager>,
        instance_id: ViewInstanceId,
        reason: SaveReason,
    ) -> Result<ForegroundDocumentSaveJob, EditorError> {
        Ok(ForegroundDocumentSaveJob::new(
            original,
            instance_id,
            reason,
        ))
    }

    pub(super) fn editor_manager_handle(
        &self,
    ) -> Result<ServiceHandle<EditorManager>, EditorError> {
        Ok(self
            .core()?
            .resolve_manager_handle::<EditorManager>(EDITOR_MANAGER_NAME)?)
    }

    pub(super) fn editor_manager(&self) -> Result<ServiceCallGuard<EditorManager>, EditorError> {
        Ok(self
            .core()?
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?)
    }

    fn core(&self) -> Result<CoreHandle, EditorError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::RuntimeUnavailable.into())
    }
}

/// Host-owned access to the one runtime project-asset service used by the editor asset manager.
///
/// This keeps the generic resolver and runtime lifetime inside the host service layer rather than
/// allowing the catalog manager to acquire unrelated runtime services.
#[derive(Clone)]
pub(super) struct EditorProjectAssetRuntimeAccess {
    core: CoreWeak,
    handle: ManagerServiceHandle<ProjectAssetManager>,
}

impl EditorProjectAssetRuntimeAccess {
    pub(super) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        Ok(Self {
            core: core.downgrade(),
            handle: project_asset_manager_handle(core)?,
        })
    }

    pub(super) fn asset_manager_identity(
        &self,
    ) -> Result<(CoreWeak, ManagerServiceHandle<dyn AssetManager>), CoreError> {
        let core = self.core()?;
        Ok((core.downgrade(), asset_manager_handle(&core)?))
    }

    pub(super) fn project_asset_manager(
        &self,
    ) -> Result<ServiceCallGuard<ProjectAssetManager>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, self.handle.clone())
    }

    pub(in crate::ui::host) fn create_import_flow(
        &self,
        jobs: EditorJobSystem,
        index: Arc<Mutex<EditorAssetIndex>>,
        logs: Arc<EditorLogService>,
    ) -> Result<EditorAssetImportFlow, CoreError> {
        // Keep the existing immediate service refusal, then release admission before queueing.
        let manager = self.project_asset_manager()?;
        drop(manager);
        EditorAssetImportFlow::new(jobs, self.core.clone(), self.handle.clone(), index, logs)
    }

    fn core(&self) -> Result<CoreHandle, CoreError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable("CoreRuntime".to_owned()))
    }
}

/// A short-lived, typed config transaction that keeps the runtime alive through rollback.
pub(super) struct EditorCapabilityConfiguration {
    _core: CoreHandle,
    config: ServiceCallGuard<dyn ConfigManager>,
}

/// A short-lived VM host capability access that resolves the runtime driver once per bootstrap.
pub(super) struct EditorVmHostCapabilityAccess {
    driver: ServiceCallGuard<PluginHostDriver>,
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
        self.config
            .get_value(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default()
    }

    pub(super) fn store_enabled_subsystems(
        &self,
        capabilities: &[String],
    ) -> Result<(), EditorError> {
        self.config
            .set_value(
                EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
                serde_json::json!(capabilities),
            )
            .map_err(|error| {
                EditorError::from(CoreError::Initialization(
                    CONFIG_MANAGER_NAME.to_owned(),
                    error.to_string(),
                ))
            })
    }

    pub(super) fn subsystem_report(&self) -> EditorSubsystemReport {
        editor_subsystem_report_from_config(
            self.config
                .get_value(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
                .and_then(|value| serde_json::from_value(value).ok()),
        )
    }
}
