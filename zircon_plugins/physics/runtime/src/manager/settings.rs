use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::{
    physics::{PhysicsSettings, PhysicsSettingsStoreError},
    scene::physics::PhysicsMaterialMetadata,
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
        let settings = core
            .and_then(CoreWeak::upgrade)
            .and_then(|core| core.load_config(crate::PHYSICS_SETTINGS_CONFIG_KEY).ok())
            .unwrap_or_else(default_settings);
        Self {
            core: Arc::new(Mutex::new(core.cloned())),
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
            if let Ok(settings) = runtime.load_config(crate::PHYSICS_SETTINGS_CONFIG_KEY) {
                *recover_lock(&self.settings) = settings;
            }
        }
        *recover_lock(&self.core) = Some(core.clone());
    }

    pub fn store_settings(
        &self,
        settings: PhysicsSettings,
    ) -> Result<(), PhysicsSettingsStoreError> {
        let backend_changed = recover_lock(&self.settings).backend != settings.backend;
        #[cfg(feature = "backend-jolt")]
        if backend_changed {
            recover_lock(&self.jolt_worlds).clear();
        }
        if backend_changed {
            recover_lock(&self.body_commands).clear();
        }
        *recover_lock(&self.settings) = settings.clone();
        *recover_lock(&self.last_backend_error) = None;
        let core = recover_lock(&self.core)
            .as_ref()
            .and_then(CoreWeak::upgrade);
        if let Some(core) = core {
            core.store_config(crate::PHYSICS_SETTINGS_CONFIG_KEY, &settings)
                .map_err(|source| PhysicsSettingsStoreError::persistence(source.to_string()))?;
        }
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
