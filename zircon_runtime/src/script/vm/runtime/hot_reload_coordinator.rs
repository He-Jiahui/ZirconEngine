use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::core::framework::script::ScriptHostValue;

use super::super::backend::{VmBackend, VmError};
use super::super::gc_bridge::{VmGcBudget, VmGcSlotStepReport, VmGcStepReport};
use super::super::handles::PluginSlotId;
use super::super::host::VmPluginHostContext;
use super::super::host_interface::VmHostInterfaceGenerationSnapshot;
use super::super::plugin::{
    migrate_vm_state_blob, VmPluginGarbageCollectionMode, VmPluginHotReloadPolicy,
    VmPluginInstance, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
};
use super::vm_plugin_slot_record::VmPluginSlotRecord;
use super::vm_plugin_slot_state::VmPluginSlotState;

pub struct HotReloadCoordinator {
    next_slot: AtomicU64,
    next_gc_frame: AtomicU64,
    lifecycle_guard: Mutex<()>,
    gc_step_guard: Mutex<()>,
    slots: Mutex<HashMap<PluginSlotId, PluginSlot>>,
    pending_gc_slots: Mutex<VecDeque<PluginSlotId>>,
}

struct PluginSlot {
    backend_name: String,
    state: VmPluginSlotState,
    generation: u32,
    source: VmPluginPackageSource,
    package: VmPluginPackage,
    host: VmPluginHostContext,
    instance: Option<Box<dyn VmPluginInstance>>,
}

struct HotReloadRollbackState {
    current_generation: u32,
    next_generation: u32,
    current_instance: Box<dyn VmPluginInstance>,
    current_host: VmPluginHostContext,
    current_state: Option<super::super::plugin::VmStateBlob>,
    registrations: VmHostInterfaceGenerationSnapshot,
}

impl PluginSlot {
    fn active(
        backend_name: String,
        generation: u32,
        source: VmPluginPackageSource,
        package: VmPluginPackage,
        host: VmPluginHostContext,
        instance: Box<dyn VmPluginInstance>,
    ) -> Self {
        Self {
            backend_name,
            state: VmPluginSlotState::Active,
            generation,
            source,
            package,
            host,
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
            next_gc_frame: AtomicU64::new(1),
            lifecycle_guard: Mutex::new(()),
            gc_step_guard: Mutex::new(()),
            slots: Mutex::new(HashMap::new()),
            pending_gc_slots: Mutex::new(VecDeque::new()),
        }
    }

    fn lock_slots(&self) -> MutexGuard<'_, HashMap<PluginSlotId, PluginSlot>> {
        self.slots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_gc_step_guard(&self) -> MutexGuard<'_, ()> {
        self.gc_step_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_lifecycle_guard(&self) -> MutexGuard<'_, ()> {
        self.lifecycle_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_pending_gc_slots(&self) -> MutexGuard<'_, VecDeque<PluginSlotId>> {
        self.pending_gc_slots
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
        let _lifecycle = self.lock_lifecycle_guard();
        let slot = PluginSlotId::new(self.next_slot.fetch_add(1, Ordering::SeqCst));
        let host = host.with_vm_owner(slot, 1);
        let mut instance = match backend.load_package(&package, &host) {
            Ok(instance) => instance,
            Err(error) => {
                host.host_interfaces.discard_slot(slot);
                return Err(error);
            }
        };
        let reflection_schema = match instance.state_schema() {
            Ok(schema) => schema,
            Err(error) => {
                host.host_interfaces.discard_slot(slot);
                return Err(error);
            }
        };
        let prepared_reflection = match host.reflection_catalog.prepare_optional_generation(
            slot,
            1,
            &package.manifest.name,
            reflection_schema.as_ref(),
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                host.host_interfaces.discard_slot(slot);
                return Err(VmError::from(error));
            }
        };
        if let Err(error) = host.install_reflection_schema(prepared_reflection.snapshot()) {
            host.host_interfaces.discard_slot(slot);
            return Err(error);
        }
        if let Err(error) = instance.activate(&host) {
            host.host_interfaces.discard_slot(slot);
            return Err(error);
        }
        if let Err(error) = host.reflection_catalog.commit_prepared(prepared_reflection) {
            let cleanup = instance.deactivate().err();
            host.host_interfaces.discard_slot(slot);
            return match cleanup {
                Some(cleanup) => Err(VmError::Operation(format!(
                    "VM reflection schema publish failed ({error}); activation cleanup failed: {cleanup}"
                ))),
                None => Err(VmError::from(error)),
            };
        }
        self.lock_slots().insert(
            slot,
            PluginSlot::active(
                backend_name.into(),
                1,
                host.package_source.clone(),
                package,
                host.clone(),
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
        let _lifecycle = self.lock_lifecycle_guard();
        let backend_name = backend_name.into();
        let (policy, mut current_instance, current_generation, next_generation, current_host) = {
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
            let next_generation = slot_entry.generation.checked_add(1).ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} generation exhausted",
                    slot.get()
                ))
            })?;
            let current_instance = slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} is already {}",
                    slot.get(),
                    slot_entry.state.label()
                ))
            })?;
            slot_entry.state = VmPluginSlotState::Reloading;
            (
                policy,
                current_instance,
                slot_entry.generation,
                next_generation,
                slot_entry.host.clone(),
            )
        };
        let next_host = host.with_vm_owner(slot, next_generation);
        let current_registrations = current_host
            .host_interfaces
            .snapshot_generation(slot, current_generation);

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

        let rollback = HotReloadRollbackState {
            current_generation,
            next_generation,
            current_instance,
            current_host,
            current_state: state,
            registrations: current_registrations,
        };

        let mut next_instance = match backend.load_package(&package, &next_host) {
            Ok(instance) => instance,
            Err(error) => {
                let error = self.rollback_hot_reload(slot, None, &next_host, rollback, error);
                return Err(error);
            }
        };
        let next_schema = match next_instance.state_schema() {
            Ok(schema) => schema,
            Err(error) => {
                let error = self.rollback_hot_reload(
                    slot,
                    Some(next_instance),
                    &next_host,
                    rollback,
                    error,
                );
                return Err(error);
            }
        };
        let prepared_reflection = match next_host.reflection_catalog.prepare_optional_generation(
            slot,
            next_generation,
            &package.manifest.name,
            next_schema.as_ref(),
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                let error = self.rollback_hot_reload(
                    slot,
                    Some(next_instance),
                    &next_host,
                    rollback,
                    VmError::from(error),
                );
                return Err(error);
            }
        };
        if let Err(error) = next_host.install_reflection_schema(prepared_reflection.snapshot()) {
            let error =
                self.rollback_hot_reload(slot, Some(next_instance), &next_host, rollback, error);
            return Err(error);
        }
        if let Err(error) = next_instance.activate(&next_host) {
            let error =
                self.rollback_hot_reload(slot, Some(next_instance), &next_host, rollback, error);
            return Err(error);
        }
        let current_state = rollback.current_state.clone();
        if let Some(state) = current_state {
            let next_state = match &next_schema {
                Some(schema) => match migrate_vm_state_blob(&state, schema) {
                    Ok(state) => state,
                    Err(error) => {
                        let error = self.rollback_hot_reload(
                            slot,
                            Some(next_instance),
                            &next_host,
                            rollback,
                            VmError::from(error),
                        );
                        return Err(error);
                    }
                },
                None => state,
            };
            if let Err(error) = next_instance.restore_state(&next_state) {
                let error = self.rollback_hot_reload(
                    slot,
                    Some(next_instance),
                    &next_host,
                    rollback,
                    error,
                );
                return Err(error);
            }
        }
        if let Err(error) = next_host
            .reflection_catalog
            .commit_prepared(prepared_reflection)
        {
            let error = self.rollback_hot_reload(
                slot,
                Some(next_instance),
                &next_host,
                rollback,
                VmError::from(error),
            );
            return Err(error);
        }

        rollback
            .current_host
            .host_interfaces
            .discard_generation(slot, rollback.current_generation);
        self.replace_slot(
            slot,
            PluginSlot::active(
                backend_name,
                next_generation,
                next_host.package_source.clone(),
                package,
                next_host,
                next_instance,
            ),
        );
        Ok(())
    }

    fn rollback_hot_reload(
        &self,
        slot: PluginSlotId,
        mut next_instance: Option<Box<dyn VmPluginInstance>>,
        next_host: &VmPluginHostContext,
        mut rollback: HotReloadRollbackState,
        primary_error: VmError,
    ) -> VmError {
        let next_deactivate_error = next_instance
            .as_mut()
            .and_then(|instance| instance.deactivate().err());
        next_host
            .host_interfaces
            .discard_generation(slot, rollback.next_generation);
        rollback
            .current_host
            .host_interfaces
            .discard_generation(slot, rollback.current_generation);
        let current_activate_error = rollback
            .current_instance
            .activate(&rollback.current_host)
            .err();
        let current_restore_error = if current_activate_error.is_none() {
            rollback
                .current_state
                .as_ref()
                .and_then(|state| rollback.current_instance.restore_state(state).err())
        } else {
            None
        };
        let rollback_succeeded =
            current_activate_error.is_none() && current_restore_error.is_none();
        rollback.current_host.host_interfaces.restore_generation(
            slot,
            rollback.current_generation,
            rollback.registrations,
        );
        self.restore_slot_instance(
            slot,
            rollback.current_instance,
            if rollback_succeeded {
                VmPluginSlotState::Active
            } else {
                VmPluginSlotState::Failed
            },
        );

        if next_deactivate_error.is_none()
            && current_activate_error.is_none()
            && current_restore_error.is_none()
        {
            return primary_error;
        }
        VmError::Operation(format!(
            "hot reload failed ({primary_error}); rollback cleanup failed: new deactivate={}, old activate={}, old restore={}",
            next_deactivate_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "ok".to_string()),
            current_activate_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "ok".to_string()),
            current_restore_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "ok".to_string()),
        ))
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
        let _lifecycle = self.lock_lifecycle_guard();
        let (host, generation) = {
            let slots = self.lock_slots();
            let slot_entry = slots.get(&slot).ok_or(VmError::MissingSlot(slot.get()))?;
            if slot_entry.state != VmPluginSlotState::Active || slot_entry.instance.is_none() {
                return Err(VmError::Operation(format!(
                    "vm plugin slot {} cannot unload while {}",
                    slot.get(),
                    slot_entry.state.label()
                )));
            }
            (slot_entry.host.clone(), slot_entry.generation)
        };
        host.reflection_catalog.validate_slot_discard(slot)?;

        let (manifest, policy, mut instance) = {
            let mut slots = self.lock_slots();
            let slot_entry = slots
                .get_mut(&slot)
                .ok_or(VmError::MissingSlot(slot.get()))?;
            let instance = slot_entry.instance.take().ok_or_else(|| {
                VmError::Operation(format!(
                    "vm plugin slot {} cannot unload while {}",
                    slot.get(),
                    slot_entry.state.label()
                ))
            })?;
            slot_entry.state = VmPluginSlotState::Unloading;
            (
                slot_entry.package.manifest.clone(),
                slot_entry.package.manifest.management.hot_reload,
                instance,
            )
        };
        let registrations = host.host_interfaces.snapshot_generation(slot, generation);
        let saved_state = match policy {
            VmPluginHotReloadPolicy::PreserveState => match instance.save_state() {
                Ok(state) => Some(state),
                Err(error) => {
                    self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
                    return Err(error);
                }
            },
            VmPluginHotReloadPolicy::Disabled | VmPluginHotReloadPolicy::Stateless => None,
        };
        if let Err(error) = instance.deactivate() {
            host.host_interfaces
                .restore_generation(slot, generation, registrations);
            self.restore_slot_instance(slot, instance, VmPluginSlotState::Failed);
            return Err(error);
        }
        if let Err(error) = host.reflection_catalog.discard_slot(slot) {
            host.host_interfaces.discard_generation(slot, generation);
            let activate_error = instance.activate(&host).err();
            let restore_error = if activate_error.is_none() {
                saved_state
                    .as_ref()
                    .and_then(|state| instance.restore_state(state).err())
            } else {
                None
            };
            let rollback_succeeded = activate_error.is_none() && restore_error.is_none();
            host.host_interfaces
                .restore_generation(slot, generation, registrations);
            self.restore_slot_instance(
                slot,
                instance,
                if rollback_succeeded {
                    VmPluginSlotState::Active
                } else {
                    VmPluginSlotState::Failed
                },
            );
            if rollback_succeeded {
                return Err(VmError::from(error));
            }
            return Err(VmError::Operation(format!(
                "VM reflection discard failed ({error}); unload rollback failed: activate={}, restore={}",
                activate_error
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "ok".to_string()),
                restore_error
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "ok".to_string())
            )));
        }
        host.host_interfaces.discard_slot(slot);
        self.lock_slots().remove(&slot);
        self.lock_pending_gc_slots()
            .retain(|pending_slot| *pending_slot != slot);
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
        let mut matches = slots
            .iter()
            .filter(|(_, entry)| {
                entry.state == VmPluginSlotState::Active
                    && entry.package.manifest.name == package_name
            })
            .map(|(slot, _)| *slot)
            .collect::<Vec<_>>();
        matches.sort_by_key(|slot| slot.get());
        let Some(slot) = matches.first().copied() else {
            return Err(VmError::Operation(format!(
                "vm plugin package {package_name} is not loaded"
            )));
        };
        if matches.len() > 1 {
            return Err(VmError::Operation(format!(
                "vm plugin package name {package_name} is ambiguous across active slots {:?}",
                matches.iter().map(|slot| slot.get()).collect::<Vec<_>>()
            )));
        }
        Ok(slot)
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

    /// Steps eligible cooperative collectors in stable slot order until the frame budget is spent.
    pub fn gc_step(&self, budget: VmGcBudget) -> Result<VmGcStepReport, VmError> {
        let _gc_step_guard = self.lock_gc_step_guard();
        let frame_index = self
            .next_gc_frame
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |frame| {
                frame.checked_add(1)
            })
            .map_err(|_| VmError::Operation("vm GC frame index exhausted".to_string()))?;
        let mut due_slots = self
            .lock_slots()
            .iter()
            .filter(|(_, entry)| gc_policy_is_due(entry, frame_index))
            .map(|(slot, _)| *slot)
            .collect::<Vec<_>>();
        due_slots.sort_by_key(|slot| slot.get());
        {
            let mut pending = self.lock_pending_gc_slots();
            for slot in due_slots {
                if !pending.contains(&slot) {
                    pending.push_back(slot);
                }
            }
        }

        let mut pause_micros = 0_u64;
        let mut slot_reports = Vec::new();
        loop {
            let remaining_micros = budget.max_micros_per_frame.saturating_sub(pause_micros);
            if remaining_micros == 0 {
                break;
            }
            let Some(slot) = self.lock_pending_gc_slots().pop_front() else {
                break;
            };
            let mut instance = {
                let mut slots = self.lock_slots();
                let Some(slot_entry) = slots.get_mut(&slot) else {
                    continue;
                };
                if !gc_policy_is_cooperative_active(slot_entry) {
                    continue;
                }
                slot_entry.instance.take().ok_or_else(|| {
                    VmError::Operation(format!(
                        "vm plugin slot {} cannot step GC while active instance is unavailable",
                        slot.get()
                    ))
                })?
            };
            let step_budget = VmGcBudget {
                max_micros_per_frame: remaining_micros,
            };
            let outcome = catch_unwind(AssertUnwindSafe(|| instance.gc_step(step_budget)));
            self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
            let outcome = match outcome {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(error)) => {
                    self.requeue_gc_slot_front(slot);
                    return Err(error);
                }
                Err(payload) => {
                    self.requeue_gc_slot_front(slot);
                    resume_unwind(payload);
                }
            };
            pause_micros = pause_micros.saturating_add(outcome.pause_micros);
            slot_reports.push(VmGcSlotStepReport {
                slot,
                budget_micros: remaining_micros,
                outcome,
            });
            if pause_micros >= budget.max_micros_per_frame {
                break;
            }
        }

        Ok(VmGcStepReport::from_slots(
            frame_index,
            budget,
            slot_reports,
        ))
    }

    fn requeue_gc_slot_front(&self, slot: PluginSlotId) {
        let mut pending = self.lock_pending_gc_slots();
        if !pending.contains(&slot) {
            pending.push_front(slot);
        }
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

fn gc_policy_is_due(slot: &PluginSlot, frame_index: u64) -> bool {
    if !gc_policy_is_cooperative_active(slot) {
        return false;
    }
    match slot
        .package
        .manifest
        .management
        .garbage_collection
        .interval_frames
    {
        None => true,
        Some(0) => false,
        Some(interval) => frame_index % interval == 0,
    }
}

fn gc_policy_is_cooperative_active(slot: &PluginSlot) -> bool {
    slot.state == VmPluginSlotState::Active
        && matches!(
            slot.package.manifest.management.garbage_collection.mode,
            VmPluginGarbageCollectionMode::Cooperative
        )
}

#[cfg(test)]
mod tests;
