use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::framework::physics::{PhysicsMaterialMetadata, PhysicsSettings};
use zircon_runtime::core::{CoreError, CoreHandle};

use crate::backend::{default_backend_name, default_simulation_mode};
use crate::manager::DefaultPhysicsManager;

impl Default for DefaultPhysicsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultPhysicsManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let settings = core
            .as_ref()
            .and_then(|core| core.load_config(crate::PHYSICS_SETTINGS_CONFIG_KEY).ok())
            .unwrap_or_else(default_settings);
        Self {
            core: Arc::new(Mutex::new(core)),
            settings: Arc::new(Mutex::new(settings)),
            default_material: PhysicsMaterialMetadata::default(),
            accumulators: Arc::new(Mutex::new(HashMap::new())),
            synced_worlds: Arc::new(Mutex::new(HashMap::new())),
            contacts: Arc::new(Mutex::new(HashMap::new())),
            trigger_pairs: Arc::new(Mutex::new(HashMap::new())),
            triggers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn attach_core(&self, core: CoreHandle) {
        if let Ok(settings) = core.load_config(crate::PHYSICS_SETTINGS_CONFIG_KEY) {
            *self
                .settings
                .lock()
                .expect("physics settings mutex poisoned") = settings;
        }
        *lock_core(&self.core) = Some(core);
    }

    pub fn store_settings(&self, settings: PhysicsSettings) -> Result<(), CoreError> {
        *self
            .settings
            .lock()
            .expect("physics settings mutex poisoned") = settings.clone();
        let core = lock_core(&self.core).clone();
        if let Some(core) = core {
            core.store_config(crate::PHYSICS_SETTINGS_CONFIG_KEY, &settings)?;
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

fn lock_core(core: &Mutex<Option<CoreHandle>>) -> MutexGuard<'_, Option<CoreHandle>> {
    core.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
