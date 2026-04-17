use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    HostRegistry, PluginSlotId, VmBackend, VmError, VmPluginInstance, VmPluginManifest,
    VmPluginPackage,
};

pub struct HotReloadCoordinator {
    backend: Arc<dyn VmBackend>,
    host: HostRegistry,
    next_slot: AtomicU64,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
}

struct PluginSlot {
    package: VmPluginPackage,
    instance: Box<dyn VmPluginInstance>,
}

impl fmt::Debug for HotReloadCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadCoordinator")
            .field("backend", &self.backend.backend_name())
            .finish()
    }
}

impl HotReloadCoordinator {
    pub fn new(backend: Arc<dyn VmBackend>, host: HostRegistry) -> Self {
        Self {
            backend,
            host,
            next_slot: AtomicU64::new(1),
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_package(&self, package: VmPluginPackage) -> Result<PluginSlotId, VmError> {
        let mut instance = self.backend.load_package(&package, self.host.clone())?;
        instance.activate(&self.host)?;
        let slot = PluginSlotId::new(self.next_slot.fetch_add(1, Ordering::SeqCst));
        self.slots
            .lock()
            .unwrap()
            .insert(slot, PluginSlot { package, instance });
        Ok(slot)
    }

    pub fn hot_reload(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError> {
        let mut slots = self.slots.lock().unwrap();
        let state = {
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let state = slot_entry.instance.save_state()?;
            slot_entry.instance.deactivate()?;
            state
        };

        let mut next_instance = self.backend.load_package(&package, self.host.clone())?;
        next_instance.activate(&self.host)?;
        next_instance.restore_state(&state)?;

        slots.insert(
            slot,
            PluginSlot {
                package,
                instance: next_instance,
            },
        );
        Ok(())
    }

    pub fn manifest(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        let slots = self.slots.lock().unwrap();
        let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
        Ok(slot_entry.package.manifest.clone())
    }
}
