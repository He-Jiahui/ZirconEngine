use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::framework::bridge::{BridgeError, PluginInterface};
use crate::plugin::extension_registry::PluginModuleId;
use crate::plugin::RuntimeExtensionRegistryError;

use super::diagnostics::{BridgeDiagnostics, BridgeDiagnosticsSnapshot};
use super::interface_id::InterfaceSlot;
use super::strong::StrongBridge;
use super::weak::WeakBridge;

#[derive(Clone)]
pub struct InterfaceExport {
    pub(crate) interface_id: String,
    pub(crate) provider: Arc<dyn Any + Send + Sync>,
}

impl InterfaceExport {
    pub(crate) fn new<T>(provider: Arc<T>) -> Self
    where
        T: PluginInterface + ?Sized,
    {
        Self {
            interface_id: T::INTERFACE_ID.to_string(),
            provider: Arc::new(provider),
        }
    }

    pub(crate) fn interface_id(&self) -> &str {
        &self.interface_id
    }
}

impl fmt::Debug for InterfaceExport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InterfaceExport")
            .field("interface_id", &self.interface_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeInterfaceStatus {
    Absent,
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeInterfaceSnapshot {
    pub slot: InterfaceSlot,
    pub interface_id: String,
    pub owner: PluginModuleId,
    pub generation: u32,
    pub provider_installed: bool,
    pub status: BridgeInterfaceStatus,
    pub diagnostics: BridgeDiagnosticsSnapshot,
}

/// Post-operation diagnostics for batch bridge changes owned by one plugin module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeOwnerTransitionReport {
    pub owner: PluginModuleId,
    pub mode: BridgeOwnerTransitionMode,
    pub affected_slots: Vec<InterfaceSlot>,
    pub snapshots: Vec<BridgeInterfaceSnapshot>,
}

impl BridgeOwnerTransitionReport {
    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.owner_transition: owner module slot {} {:?} affected {} interface(s): [{}]",
            self.owner.raw(),
            self.mode,
            self.affected_slots.len(),
            self.snapshots
                .iter()
                .map(|snapshot| format!(
                    "`{}`@slot{} generation={} provider_installed={} status={:?}",
                    snapshot.interface_id,
                    snapshot.slot.raw(),
                    snapshot.generation,
                    snapshot.provider_installed,
                    snapshot.status
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeOwnerTransitionMode {
    Activate,
    Disable,
    Deactivate,
}

#[derive(Debug)]
pub struct BridgeEntry {
    interface_id: String,
    provider: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
    /// Generation parity is the bridge enablement contract:
    /// even generations are enabled, odd generations are disabled.
    generation: AtomicU32,
    owner: PluginModuleId,
    diagnostics: BridgeDiagnostics,
}

impl BridgeEntry {
    fn new(
        interface_id: String,
        provider: Arc<dyn Any + Send + Sync>,
        owner: PluginModuleId,
    ) -> Self {
        Self {
            interface_id,
            provider: Mutex::new(Some(provider)),
            generation: AtomicU32::new(0),
            owner,
            diagnostics: BridgeDiagnostics::default(),
        }
    }

    pub fn interface_id(&self) -> &str {
        &self.interface_id
    }

    pub fn owner(&self) -> PluginModuleId {
        self.owner
    }

    pub fn generation(&self) -> u32 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn is_enabled(&self) -> bool {
        self.generation() % 2 == 0 && self.provider_installed()
    }

    pub fn provider_installed(&self) -> bool {
        self.provider.lock().unwrap().is_some()
    }

    pub fn diagnostics(&self) -> BridgeDiagnosticsSnapshot {
        self.diagnostics.snapshot()
    }

    pub fn status(&self) -> BridgeInterfaceStatus {
        if self.is_enabled() {
            BridgeInterfaceStatus::Enabled
        } else {
            BridgeInterfaceStatus::Disabled
        }
    }

    pub(crate) fn record_enabled_call(&self) {
        self.diagnostics.record_enabled_call();
    }

    pub(crate) fn record_not_enabled_call(&self) {
        self.diagnostics.record_not_enabled_call();
    }

    fn provider<T>(&self) -> Result<(u32, Arc<T>), BridgeError>
    where
        T: PluginInterface + ?Sized,
    {
        let generation = self.generation();
        if generation % 2 != 0 {
            return Err(BridgeError::NotEnabled);
        }

        let provider = self
            .provider
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or(BridgeError::NotEnabled)?;
        let provider = provider
            .downcast::<Arc<T>>()
            .map_err(|_| BridgeError::NotEnabled)?;
        Ok((generation, (*provider).clone()))
    }

    fn set_enabled(&self, enabled: bool) {
        let mut current = self.generation();
        loop {
            let currently_enabled = current % 2 == 0;
            if currently_enabled == enabled {
                return;
            }

            match self.generation.compare_exchange(
                current,
                current.wrapping_add(1),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(next) => current = next,
            }
        }
    }

    fn deactivate(&self) {
        self.set_enabled(false);
        *self.provider.lock().unwrap() = None;
    }

    fn replace_provider<T>(&self, provider: Arc<T>)
    where
        T: PluginInterface + ?Sized,
    {
        *self.provider.lock().unwrap() = Some(Arc::new(provider));
        if self.generation() % 2 == 0 {
            self.generation.fetch_add(2, Ordering::AcqRel);
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrozenBridgeTable {
    inner: Arc<FrozenBridgeTableInner>,
}

#[derive(Debug)]
struct FrozenBridgeTableInner {
    entries: Box<[BridgeEntry]>,
    slots_by_interface: HashMap<String, InterfaceSlot>,
}

impl FrozenBridgeTable {
    pub(crate) fn from_exports(
        exports: impl IntoIterator<Item = (PluginModuleId, String, InterfaceExport)>,
    ) -> Self {
        let mut entries = Vec::new();
        let mut slots_by_interface = HashMap::new();
        for (owner, interface_id, export) in exports {
            let slot = InterfaceSlot::from_raw(entries.len() as u32);
            slots_by_interface.insert(interface_id.clone(), slot);
            entries.push(BridgeEntry::new(interface_id, export.provider, owner));
        }

        Self {
            inner: Arc::new(FrozenBridgeTableInner {
                entries: entries.into_boxed_slice(),
                slots_by_interface,
            }),
        }
    }

    pub fn resolve_slot(&self, interface_id: &str) -> Option<InterfaceSlot> {
        self.inner.slots_by_interface.get(interface_id).copied()
    }

    pub fn entry(&self, slot: InterfaceSlot) -> Option<&BridgeEntry> {
        self.inner.entries.get(slot.index())
    }

    pub fn entries(&self) -> &[BridgeEntry] {
        &self.inner.entries
    }

    pub fn diagnostics(&self, slot: InterfaceSlot) -> Option<BridgeDiagnosticsSnapshot> {
        self.entry(slot).map(BridgeEntry::diagnostics)
    }

    pub fn interface_status(&self, interface_id: &str) -> BridgeInterfaceStatus {
        let Some(slot) = self.resolve_slot(interface_id) else {
            return BridgeInterfaceStatus::Absent;
        };
        let Some(entry) = self.entry(slot) else {
            return BridgeInterfaceStatus::Absent;
        };
        entry.status()
    }

    pub fn interface_snapshots(&self) -> Vec<BridgeInterfaceSnapshot> {
        self.inner
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| self.snapshot_for_entry(index, entry))
            .collect()
    }

    pub fn interface_snapshot(&self, slot: InterfaceSlot) -> Option<BridgeInterfaceSnapshot> {
        self.entry(slot)
            .map(|entry| self.snapshot_for_entry(slot.index(), entry))
    }

    pub fn interface_snapshot_by_id(&self, interface_id: &str) -> Option<BridgeInterfaceSnapshot> {
        self.resolve_slot(interface_id)
            .and_then(|slot| self.interface_snapshot(slot))
    }

    pub fn interface_snapshots_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> Vec<BridgeInterfaceSnapshot> {
        self.inner
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.owner() == owner)
            .map(|(index, entry)| self.snapshot_for_entry(index, entry))
            .collect()
    }

    pub fn resolve_strong<T>(&self) -> Result<StrongBridge<T>, RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        let slot = self.resolve_slot(T::INTERFACE_ID).ok_or_else(|| {
            RuntimeExtensionRegistryError::MissingPluginInterface(T::INTERFACE_ID.to_string())
        })?;
        let (_, provider) = self
            .entry(slot)
            .expect("resolved slot")
            .provider::<T>()
            .map_err(|_| {
                RuntimeExtensionRegistryError::MissingPluginInterface(T::INTERFACE_ID.to_string())
            })?;
        Ok(StrongBridge::new(provider))
    }

    pub fn resolve_weak<T>(&self) -> WeakBridge<T>
    where
        T: PluginInterface + ?Sized,
    {
        WeakBridge::new(self.clone(), self.resolve_slot(T::INTERFACE_ID))
    }

    pub fn set_enabled(
        &self,
        slot: InterfaceSlot,
        enabled: bool,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let entry = self.entry(slot).ok_or_else(|| {
            RuntimeExtensionRegistryError::MissingPluginInterface(format!("slot:{}", slot.raw()))
        })?;
        entry.set_enabled(enabled);
        Ok(())
    }

    pub fn set_owner_enabled(&self, owner: PluginModuleId, enabled: bool) -> Vec<InterfaceSlot> {
        self.set_owner_enabled_slots(owner, enabled)
    }

    pub fn set_owner_enabled_with_report(
        &self,
        owner: PluginModuleId,
        enabled: bool,
    ) -> BridgeOwnerTransitionReport {
        let affected_slots = self.set_owner_enabled_slots(owner, enabled);
        let mode = if enabled {
            BridgeOwnerTransitionMode::Activate
        } else {
            BridgeOwnerTransitionMode::Disable
        };
        self.owner_transition_report(owner, mode, affected_slots)
    }

    fn set_owner_enabled_slots(&self, owner: PluginModuleId, enabled: bool) -> Vec<InterfaceSlot> {
        let mut affected_slots = Vec::new();
        for (index, entry) in self.inner.entries.iter().enumerate() {
            if entry.owner() != owner {
                continue;
            }

            entry.set_enabled(enabled);
            affected_slots.push(InterfaceSlot::from_raw(index as u32));
        }
        affected_slots
    }

    pub fn activate_owner(&self, owner: PluginModuleId) -> Vec<InterfaceSlot> {
        self.set_owner_enabled(owner, true)
    }

    pub fn activate_owner_with_report(&self, owner: PluginModuleId) -> BridgeOwnerTransitionReport {
        self.set_owner_enabled_with_report(owner, true)
    }

    pub fn deactivate_owner(&self, owner: PluginModuleId) -> Vec<InterfaceSlot> {
        self.deactivate_owner_slots(owner)
    }

    pub fn deactivate_owner_with_report(
        &self,
        owner: PluginModuleId,
    ) -> BridgeOwnerTransitionReport {
        let affected_slots = self.deactivate_owner_slots(owner);
        self.owner_transition_report(owner, BridgeOwnerTransitionMode::Deactivate, affected_slots)
    }

    fn deactivate_owner_slots(&self, owner: PluginModuleId) -> Vec<InterfaceSlot> {
        let mut affected_slots = Vec::new();
        for (index, entry) in self.inner.entries.iter().enumerate() {
            if entry.owner() != owner {
                continue;
            }

            entry.deactivate();
            affected_slots.push(InterfaceSlot::from_raw(index as u32));
        }
        affected_slots
    }

    pub fn replace_provider<T>(
        &self,
        slot: InterfaceSlot,
        provider: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        let entry = self.entry(slot).ok_or_else(|| {
            RuntimeExtensionRegistryError::MissingPluginInterface(T::INTERFACE_ID.to_string())
        })?;
        entry.replace_provider(provider);
        Ok(())
    }

    pub fn reload_provider<T>(
        &self,
        slot: InterfaceSlot,
        provider: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.replace_provider(slot, provider)
    }

    pub(crate) fn provider<T>(&self, slot: InterfaceSlot) -> Result<(u32, Arc<T>), BridgeError>
    where
        T: PluginInterface + ?Sized,
    {
        self.entry(slot).ok_or(BridgeError::Absent)?.provider::<T>()
    }

    pub(crate) fn record_enabled_call(&self, slot: InterfaceSlot) {
        if let Some(entry) = self.entry(slot) {
            entry.record_enabled_call();
        }
    }

    pub(crate) fn record_not_enabled_call(&self, slot: InterfaceSlot) {
        if let Some(entry) = self.entry(slot) {
            entry.record_not_enabled_call();
        }
    }

    fn snapshot_for_entry(&self, index: usize, entry: &BridgeEntry) -> BridgeInterfaceSnapshot {
        BridgeInterfaceSnapshot {
            slot: InterfaceSlot::from_raw(index as u32),
            interface_id: entry.interface_id().to_string(),
            owner: entry.owner(),
            generation: entry.generation(),
            provider_installed: entry.provider_installed(),
            status: entry.status(),
            diagnostics: entry.diagnostics(),
        }
    }

    fn owner_transition_report(
        &self,
        owner: PluginModuleId,
        mode: BridgeOwnerTransitionMode,
        affected_slots: Vec<InterfaceSlot>,
    ) -> BridgeOwnerTransitionReport {
        let snapshots = affected_slots
            .iter()
            .filter_map(|slot| {
                self.entry(*slot)
                    .map(|entry| self.snapshot_for_entry(slot.index(), entry))
            })
            .collect();
        BridgeOwnerTransitionReport {
            owner,
            mode,
            affected_slots,
            snapshots,
        }
    }
}
