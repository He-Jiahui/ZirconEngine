use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::bridge::{BridgeError, PluginInterface};
use crate::plugin::extension_registry::PluginModuleId;
use crate::plugin::RuntimeExtensionRegistryError;

use super::diagnostics::{BridgeDiagnostics, BridgeDiagnosticsSnapshot};
use super::interface_id::InterfaceSlot;
use super::strong::StrongBridge;
use super::weak::WeakBridge;

mod reports;

pub use self::reports::{
    BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot, BridgeInterfaceStatus,
    BridgeOwnerTransitionMode, BridgeOwnerTransitionReport, BridgeTableDiagnosticsSummary,
};

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

    pub(crate) fn provider(&self) -> Arc<dyn Any + Send + Sync> {
        self.provider.clone()
    }
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
        self.snapshot_state().status == BridgeInterfaceStatus::Enabled
    }

    pub fn provider_installed(&self) -> bool {
        self.lock_provider().is_some()
    }

    pub fn diagnostics(&self) -> BridgeDiagnosticsSnapshot {
        self.diagnostics.snapshot()
    }

    pub fn status(&self) -> BridgeInterfaceStatus {
        self.snapshot_state().status
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
            .lock_provider()
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
        *self.lock_provider() = None;
    }

    fn replace_provider<T>(&self, provider: Arc<T>)
    where
        T: PluginInterface + ?Sized,
    {
        self.replace_erased_provider(Arc::new(provider));
    }

    fn replace_erased_provider(&self, provider: Arc<dyn Any + Send + Sync>) {
        *self.lock_provider() = Some(provider);
        if self.generation() % 2 == 0 {
            self.generation.fetch_add(2, Ordering::AcqRel);
        }
    }

    fn restore_provider(&self, provider: Arc<dyn Any + Send + Sync>) {
        *self.lock_provider() = Some(provider);
        self.set_enabled(true);
    }

    fn lock_provider(&self) -> MutexGuard<'_, Option<Arc<dyn Any + Send + Sync>>> {
        self.provider
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn snapshot_state(&self) -> BridgeEntrySnapshotState {
        let generation = self.generation();
        let provider_installed = self.provider_installed();
        let status = BridgeInterfaceStatus::from_installed_entry(generation, provider_installed);
        BridgeEntrySnapshotState {
            generation,
            provider_installed,
            status,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BridgeEntrySnapshotState {
    generation: u32,
    provider_installed: bool,
    status: BridgeInterfaceStatus,
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

    pub fn diagnostics_summary(&self) -> BridgeTableDiagnosticsSummary {
        self.summarize_entries(self.inner.entries.iter().enumerate())
    }

    pub fn diagnostics_summary_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> BridgeTableDiagnosticsSummary {
        self.summarize_entries(
            self.inner
                .entries
                .iter()
                .enumerate()
                .filter(move |(_, entry)| entry.owner() == owner),
        )
    }

    pub fn diagnostics_matrix(&self) -> BridgeDiagnosticsMatrix {
        BridgeDiagnosticsMatrix::from_rows(self.interface_snapshots())
    }

    pub fn diagnostics_matrix_owned_by(&self, owner: PluginModuleId) -> BridgeDiagnosticsMatrix {
        BridgeDiagnosticsMatrix::from_rows(self.interface_snapshots_owned_by(owner))
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

    pub(crate) fn restore_owner_exports_with_report(
        &self,
        owner: PluginModuleId,
        exports: impl IntoIterator<Item = (String, InterfaceExport)>,
    ) -> BridgeOwnerTransitionReport {
        let mut affected_slots = Vec::new();
        for (interface_id, export) in exports {
            let Some(slot) = self.resolve_slot(&interface_id) else {
                continue;
            };
            let Some(entry) = self.entry(slot) else {
                continue;
            };
            if entry.owner() != owner {
                continue;
            }

            entry.restore_provider(export.provider());
            affected_slots.push(slot);
        }
        affected_slots.sort_by_key(|slot| slot.raw());
        affected_slots.dedup();
        self.owner_transition_report(owner, BridgeOwnerTransitionMode::Activate, affected_slots)
    }

    pub(crate) fn reload_owner_exports_with_report(
        &self,
        owner: PluginModuleId,
        exports: impl IntoIterator<Item = (String, InterfaceExport)>,
    ) -> BridgeOwnerTransitionReport {
        let mut affected_slots = Vec::new();
        for (interface_id, export) in exports {
            let Some(slot) = self.resolve_slot(&interface_id) else {
                continue;
            };
            let Some(entry) = self.entry(slot) else {
                continue;
            };
            if entry.owner() != owner {
                continue;
            }

            entry.replace_erased_provider(export.provider());
            affected_slots.push(slot);
        }
        affected_slots.sort_by_key(|slot| slot.raw());
        affected_slots.dedup();
        self.owner_transition_report(owner, BridgeOwnerTransitionMode::Reload, affected_slots)
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
        let state = entry.snapshot_state();
        BridgeInterfaceSnapshot {
            slot: InterfaceSlot::from_raw(index as u32),
            interface_id: entry.interface_id().to_string(),
            owner: entry.owner(),
            generation: state.generation,
            provider_installed: state.provider_installed,
            status: state.status,
            diagnostics: entry.diagnostics(),
        }
    }

    fn summarize_entries<'a>(
        &self,
        entries: impl IntoIterator<Item = (usize, &'a BridgeEntry)>,
    ) -> BridgeTableDiagnosticsSummary {
        let mut summary = BridgeTableDiagnosticsSummary::default();
        for (index, entry) in entries {
            summary.record_snapshot(&self.snapshot_for_entry(index, entry));
        }
        summary
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

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    trait PoisonBridge: Send + Sync {
        fn sample(&self) -> i32;
    }

    impl PluginInterface for dyn PoisonBridge {
        const INTERFACE_ID: &'static str = "test.poison.bridge.v1";
    }

    struct PoisonBridgeProvider {
        value: i32,
    }

    impl PoisonBridge for PoisonBridgeProvider {
        fn sample(&self) -> i32 {
            self.value
        }
    }

    #[test]
    fn bridge_entry_provider_accessors_recover_poisoned_provider_lock() {
        let entry = BridgeEntry::new(
            <dyn PoisonBridge as PluginInterface>::INTERFACE_ID.to_string(),
            erased_provider(7),
            PluginModuleId::from_raw(7),
        );

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = entry.provider.lock().unwrap();
            panic!("poison bridge entry provider lock");
        }));

        assert!(entry.provider_installed());
        assert_eq!(entry.status(), BridgeInterfaceStatus::Enabled);
        let (_, provider) = entry
            .provider::<dyn PoisonBridge>()
            .expect("provider should recover after poison");
        assert_eq!(provider.sample(), 7);

        entry.deactivate();
        assert!(!entry.provider_installed());
        assert_eq!(entry.status(), BridgeInterfaceStatus::Disabled);

        entry.restore_provider(erased_provider(11));
        let (_, provider) = entry
            .provider::<dyn PoisonBridge>()
            .expect("restored provider should recover after poison");
        assert_eq!(provider.sample(), 11);

        let replacement: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value: 13 });
        entry.replace_provider(replacement);
        let (_, provider) = entry
            .provider::<dyn PoisonBridge>()
            .expect("replaced provider should recover after poison");
        assert_eq!(provider.sample(), 13);
    }

    fn erased_provider(value: i32) -> Arc<dyn Any + Send + Sync> {
        let provider: Arc<dyn PoisonBridge> = Arc::new(PoisonBridgeProvider { value });
        Arc::new(provider)
    }
}
