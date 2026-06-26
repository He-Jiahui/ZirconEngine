use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::core::framework::script::ScriptHostValue;

use super::super::backend::{VmBackend, VmError};
use super::super::handles::PluginSlotId;
use super::super::host::VmPluginHostContext;
use super::super::plugin::{
    VmPluginHotReloadPolicy, VmPluginInstance, VmPluginManifest, VmPluginPackage,
    VmPluginPackageSource,
};
use super::vm_plugin_slot_record::VmPluginSlotRecord;
use super::vm_plugin_slot_state::VmPluginSlotState;

pub struct HotReloadCoordinator {
    next_slot: AtomicU64,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
}

struct PluginSlot {
    backend_name: String,
    state: VmPluginSlotState,
    generation: u64,
    source: VmPluginPackageSource,
    package: VmPluginPackage,
    instance: Option<Box<dyn VmPluginInstance>>,
}

impl PluginSlot {
    fn active(
        backend_name: String,
        generation: u64,
        source: VmPluginPackageSource,
        package: VmPluginPackage,
        instance: Box<dyn VmPluginInstance>,
    ) -> Self {
        Self {
            backend_name,
            state: VmPluginSlotState::Active,
            generation,
            source,
            package,
            instance: Some(instance),
        }
    }

    fn record(&self, slot: PluginSlotId) -> VmPluginSlotRecord {
        VmPluginSlotRecord {
            slot,
            backend_name: self.backend_name.clone(),
            state: self.state,
            generation: self.generation,
            source: self.source.clone(),
            manifest: self.package.manifest.clone(),
            management: self.package.manifest.management.clone(),
        }
    }
}

impl fmt::Debug for HotReloadCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadCoordinator")
            .field("slot_count", &self.lock_slots().len())
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

    fn lock_slots(&self) -> MutexGuard<'_, HashMap<PluginSlotId, PluginSlot>> {
        self.slots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
        self.lock_slots().insert(
            slot,
            PluginSlot::active(
                backend_name.into(),
                1,
                host.package_source.clone(),
                package,
                instance,
            ),
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
        let backend_name = backend_name.into();
        let (policy, mut current_instance, next_generation) = {
            let mut slots = self.lock_slots();
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let policy = slot_entry.package.manifest.management.hot_reload;
            if matches!(policy, VmPluginHotReloadPolicy::Disabled) {
                return Err(VmError::Operation(format!(
                    "vm plugin slot {} does not allow hot reload",
                    slot.get()
                )));
            }
            let current_instance = slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} is already {}",
                    slot.get(),
                    slot_entry.state.label()
                ))
            })?;
            slot_entry.state = VmPluginSlotState::Reloading;
            (policy, current_instance, slot_entry.generation + 1)
        };

        let state = match policy {
            VmPluginHotReloadPolicy::Disabled => unreachable!("disabled policy returned above"),
            VmPluginHotReloadPolicy::Stateless => None,
            VmPluginHotReloadPolicy::PreserveState => match current_instance.save_state() {
                Ok(state) => Some(state),
                Err(error) => {
                    self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Active);
                    return Err(error);
                }
            },
        };
        if let Err(error) = current_instance.deactivate() {
            self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
            return Err(error);
        }

        let mut next_instance = match backend.load_package(&package, host) {
            Ok(instance) => instance,
            Err(error) => {
                self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
                return Err(error);
            }
        };
        if let Err(error) = next_instance.activate(host) {
            self.restore_slot_instance(slot, current_instance, VmPluginSlotState::Failed);
            return Err(error);
        }
        if let Some(state) = &state {
            if let Err(error) = next_instance.restore_state(state) {
                self.replace_slot(
                    slot,
                    PluginSlot {
                        backend_name,
                        state: VmPluginSlotState::Failed,
                        generation: next_generation,
                        source: host.package_source.clone(),
                        package,
                        instance: Some(next_instance),
                    },
                );
                return Err(error);
            }
        }

        self.replace_slot(
            slot,
            PluginSlot::active(
                backend_name,
                next_generation,
                host.package_source.clone(),
                package,
                next_instance,
            ),
        );
        Ok(())
    }

    fn restore_slot_instance(
        &self,
        slot: PluginSlotId,
        instance: Box<dyn VmPluginInstance>,
        state: VmPluginSlotState,
    ) {
        let mut slots = self.lock_slots();
        if let Some(slot_entry) = slots.get_mut(&slot) {
            slot_entry.instance = Some(instance);
            slot_entry.state = state;
        }
    }

    fn replace_slot(&self, slot: PluginSlotId, slot_entry: PluginSlot) {
        self.lock_slots().insert(slot, slot_entry);
    }

    pub fn unload_slot(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        let mut slot_entry = {
            let mut slots = self.lock_slots();
            if let Some(slot_entry) = slots.get(&slot) {
                if slot_entry.instance.is_none() {
                    return Err(VmError::Operation(format!(
                        "vm plugin slot {} cannot unload while {}",
                        slot.get(),
                        slot_entry.state.label()
                    )));
                }
            }
            slots
                .remove(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?
        };
        let manifest = slot_entry.package.manifest.clone();
        if let Some(mut instance) = slot_entry.instance.take() {
            instance.deactivate()?;
        }
        Ok(manifest)
    }

    pub fn manifest(&self, slot: PluginSlotId) -> Result<VmPluginManifest, VmError> {
        Ok(self.slot(slot)?.manifest)
    }

    pub fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        let slots = self.lock_slots();
        let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
        Ok(slot_entry.record(slot))
    }

    pub fn slot_for_package_name(&self, package_name: &str) -> Result<PluginSlotId, VmError> {
        let slots = self.lock_slots();
        slots
            .iter()
            .filter(|(_, entry)| {
                entry.state == VmPluginSlotState::Active
                    && entry.package.manifest.name == package_name
            })
            .map(|(slot, _)| *slot)
            .min_by_key(|slot| slot.get())
            .ok_or_else(|| {
                VmError::Operation(format!("vm plugin package {package_name} is not loaded"))
            })
    }

    pub fn call_slot_export(
        &self,
        slot: PluginSlotId,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let mut instance = {
            let mut slots = self.lock_slots();
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            if slot_entry.state != VmPluginSlotState::Active {
                return Err(VmError::Operation(format!(
                    "vm plugin slot {} cannot call export while {}",
                    slot.get(),
                    slot_entry.state.label()
                )));
            }
            slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} cannot call export while active instance is unavailable",
                    slot.get()
                ))
            })?
        };

        let result = instance.call_export(module_name, export_name, arguments);
        self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
        result
    }

    pub fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        let mut records = self
            .lock_slots()
            .iter()
            .map(|(slot, entry)| entry.record(*slot))
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.slot.get());
        records
    }
}

#[cfg(test)]
mod tests;
