use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::framework::{
    physics::{PhysicsSettings, PhysicsSettingsStoreError},
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::manager::{
    config_manager_handle, resolve_manager_service, CONFIG_MANAGER_NAME,
};
use zircon_runtime::core::CoreWeak;

use crate::backend::{default_backend_name, default_simulation_mode};
use crate::manager::DefaultPhysicsManager;

use super::poison_recovery::recover_lock;

impl Default for DefaultPhysicsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultPhysicsManager {
    pub fn new(core: Option<&CoreWeak>) -> Self {
        let config_manager = core
            .and_then(CoreWeak::upgrade)
            .and_then(|core| config_manager_handle(&core).ok());
        let settings = core
            .zip(config_manager.as_ref())
            .and_then(|(weak_core, handle)| {
                weak_core
                    .upgrade()
                    .and_then(|core| resolve_manager_service(&core, handle.clone()).ok())
            })
            .and_then(|config| {
                config
                    .get_value(crate::PHYSICS_SETTINGS_CONFIG_KEY)
                    .and_then(|value| serde_json::from_value(value).ok())
            })
            .unwrap_or_else(default_settings);
        Self {
            core: Arc::new(Mutex::new(core.cloned())),
            config_manager: Arc::new(Mutex::new(config_manager)),
            settings: Arc::new(Mutex::new(settings)),
            default_material: PhysicsMaterialMetadata::default(),
            accumulators: Arc::new(Mutex::new(HashMap::new())),
            synced_worlds: Arc::new(Mutex::new(HashMap::new())),
            contacts: Arc::new(Mutex::new(HashMap::new())),
            trigger_pairs: Arc::new(Mutex::new(HashMap::new())),
            triggers: Arc::new(Mutex::new(HashMap::new())),
            body_commands: Arc::new(Mutex::new(HashMap::new())),
            last_backend_error: Arc::new(Mutex::new(None)),
            #[cfg(feature = "backend-jolt")]
            jolt_worlds: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn attach_core(&self, core: &CoreWeak) {
        if let Some(runtime) = core.upgrade() {
            let config_manager = config_manager_handle(&runtime).ok();
            if let Some(handle) = config_manager.as_ref() {
                if let Ok(config) = resolve_manager_service(&runtime, handle.clone()) {
                    if let Some(settings) = config
                        .get_value(crate::PHYSICS_SETTINGS_CONFIG_KEY)
                        .and_then(|value| serde_json::from_value(value).ok())
                    {
                        *recover_lock(&self.settings) = settings;
                    }
                }
            }
            *recover_lock(&self.config_manager) = config_manager;
        }
        *recover_lock(&self.core) = Some(core.clone());
    }

    pub fn store_settings(
        &self,
        settings: PhysicsSettings,
    ) -> Result<(), PhysicsSettingsStoreError> {
        let backend_changed = recover_lock(&self.settings).backend != settings.backend;
        let core = recover_lock(&self.core)
            .as_ref()
            .and_then(CoreWeak::upgrade);
        let config_manager = recover_lock(&self.config_manager).clone();
        if let Some(core) = core.as_ref() {
            let Some(handle) = config_manager else {
                return Err(PhysicsSettingsStoreError::persistence(format!(
                    "missing {CONFIG_MANAGER_NAME}"
                )));
            };
            let config = resolve_manager_service(core, handle)
                .map_err(|source| PhysicsSettingsStoreError::persistence(source.to_string()))?;
            let value = serde_json::to_value(&settings)
                .map_err(|source| PhysicsSettingsStoreError::persistence(source.to_string()))?;
            config
                .set_value(crate::PHYSICS_SETTINGS_CONFIG_KEY, value)
                .map_err(|source| PhysicsSettingsStoreError::persistence(source.to_string()))?;
        }
        #[cfg(feature = "backend-jolt")]
        if backend_changed {
            recover_lock(&self.jolt_worlds).clear();
        }
        if backend_changed {
            recover_lock(&self.body_commands).clear();
        }
        *recover_lock(&self.settings) = settings.clone();
        *recover_lock(&self.last_backend_error) = None;
        Ok(())
    }
}

pub(super) fn default_settings() -> PhysicsSettings {
    PhysicsSettings {
        backend: default_backend_name(),
        simulation_mode: default_simulation_mode(),
        ..PhysicsSettings::default()
    }
}
