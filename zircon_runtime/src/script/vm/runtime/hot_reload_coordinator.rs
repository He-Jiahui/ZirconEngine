use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::super::backend::{VmBackend, VmError};
use super::super::handles::PluginSlotId;
use super::super::host::VmPluginHostContext;
use super::super::plugin::{
    VmPluginInstance, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
};
use super::vm_plugin_slot_record::VmPluginSlotRecord;

pub struct HotReloadCoordinator {
    next_slot: AtomicU64,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
}

struct PluginSlot {
    backend_name: String,
    source: VmPluginPackageSource,
    package: VmPluginPackage,
    instance: Box<dyn VmPluginInstance>,
}

impl fmt::Debug for HotReloadCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadCoordinator")
            .field("slot_count", &self.slots.lock().unwrap().len())
            .finish()
    }
}

impl HotReloadCoordinator {
    pub fn new() -> Self {
        Self {
            next_slot: AtomicU64::new(1),
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_package(
        &self,
        backend_name: impl Into<String>,
        backend: &dyn VmBackend,
        package: VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<PluginSlotId, VmError> {
        let mut instance = backend.load_package(&package, host)?;
        instance.activate(host)?;
        let slot = PluginSlotId::new(self.next_slot.fetch_add(1, Ordering::SeqCst));
        self.slots.lock().unwrap().insert(
            slot,
            PluginSlot {
                backend_name: backend_name.into(),
                source: host.package_source.clone(),
                package,
                instance,
            },
        );
        Ok(slot)
    }

    pub fn hot_reload(
        &self,
        slot: PluginSlotId,
        backend_name: impl Into<String>,
        backend: &dyn VmBackend,
        package: VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<(), VmError> {
        let mut slots = self.slots.lock().unwrap();
        let state = {
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let state = slot_entry.instance.save_state()?;
            slot_entry.instance.deactivate()?;
            state
        };

        let mut next_instance = backend.load_package(&package, host)?;
        next_instance.activate(host)?;
        next_instance.restore_state(&state)?;

        slots.insert(
            slot,
            PluginSlot {
                backend_name: backend_name.into(),
                source: host.package_source.clone(),
                package,
                instance: next_instance,
            },
        );
        Ok(())
    }

    pub fn unload_slot(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        let mut slot_entry = self
            .slots
            .lock()
            .unwrap()
            .remove(&slot)
            .ok_or(VmError::MissingSlot(slot.get()))?;
        let manifest = slot_entry.package.manifest.clone();
        slot_entry.instance.deactivate()?;
        Ok(manifest)
    }

    pub fn manifest(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        Ok(self.slot(slot)?.manifest)
    }

    pub fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        let slots = self.slots.lock().unwrap();
        let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
        Ok(VmPluginSlotRecord {
            slot,
            backend_name: slot_entry.backend_name.clone(),
            source: slot_entry.source.clone(),
            manifest: slot_entry.package.manifest.clone(),
        })
    }

    pub fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        let mut records = self
            .slots
            .lock()
            .unwrap()
            .iter()
            .map(|(slot, entry)| VmPluginSlotRecord {
                slot: *slot,
                backend_name: entry.backend_name.clone(),
                source: entry.source.clone(),
                manifest: entry.package.manifest.clone(),
            })
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.slot.get());
        records
    }
}
