use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::core::framework::bridge::{
    BridgeDiagnostics, BridgeDiagnosticsSnapshot, BridgeError, BridgeInterfaceStatus,
    BridgeInvocationTable, BridgeOwnerTransitionMode, InterfaceSlot, PluginInterface, StrongBridge,
};
use crate::plugin::extension_registry::PluginModuleId;
use crate::plugin::RuntimeExtensionRegistryError;

use super::weak::WeakBridge;

mod reports;

pub use self::reports::{
    BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot, BridgeOwnerTransitionReport,
    BridgeTableDiagnosticsSummary,
};

/// 注册时封装类型化 trait provider；冻结表按接口 ID 建立固定 slot，热更新只替换 slot 内的 provider。
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

/// 一个 owner 的接口槽。代际与 provider 同次原子发布，弱句柄才可按代际判断缓存有效性。
pub struct BridgeEntry {
    interface_id: String,
    state: ArcSwap<BridgeEntryState>,
    owner: PluginModuleId,
    diagnostics: BridgeDiagnostics,
}

#[derive(Clone)]
struct BridgeEntryState {
    /// Generation parity is the bridge enablement contract:
    /// even generations are enabled, odd generations are disabled.
    generation: u32,
    provider: Option<Arc<dyn Any + Send + Sync>>,
}

impl BridgeEntry {
    fn new(
        interface_id: String,
        provider: Arc<dyn Any + Send + Sync>,
        owner: PluginModuleId,
    ) -> Self {
        Self {
            interface_id,
            state: ArcSwap::from_pointee(BridgeEntryState {
                generation: 0,
                provider: Some(provider),
            }),
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
        self.state.load().generation
    }

    pub fn is_enabled(&self) -> bool {
        self.snapshot_state().status == BridgeInterfaceStatus::Enabled
    }

    pub fn provider_installed(&self) -> bool {
        self.state.load().provider.is_some()
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
        let state = self.state.load();
        if state.generation % 2 != 0 {
            return Err(BridgeError::NotEnabled);
        }

        let provider = state
            .provider
            .as_ref()
            .cloned()
            .ok_or(BridgeError::NotEnabled)?;
        let provider = provider
            .downcast::<Arc<T>>()
            .map_err(|_| BridgeError::NotEnabled)?;
        Ok((state.generation, (*provider).clone()))
    }

    // Disable 保留 provider 以便恢复；Deactivate 才清空它。启用位切换后，先前的弱缓存失效。
    fn set_enabled(&self, enabled: bool) {
        self.state.rcu(|current| {
            let currently_enabled = current.generation % 2 == 0;
            if currently_enabled == enabled {
                return Arc::clone(current);
            }
            Arc::new(BridgeEntryState {
                generation: current.generation.wrapping_add(1),
                provider: current.provider.clone(),
            })
        });
    }

    fn deactivate(&self) {
        self.state.rcu(|current| {
            let generation = if current.generation % 2 == 0 {
                current.generation.wrapping_add(1)
            } else {
                current.generation
            };
            Arc::new(BridgeEntryState {
                generation,
                provider: None,
            })
        });
    }

    fn replace_provider<T>(&self, provider: Arc<T>)
    where
        T: PluginInterface + ?Sized,
    {
        self.replace_erased_provider(Arc::new(provider));
    }

    // 生命周期热更新沿原 slot 发布新 provider；启用态跳过一个奇数代次，禁用态等待后续启用。
    fn replace_erased_provider(&self, provider: Arc<dyn Any + Send + Sync>) {
        self.state.rcu(|current| {
            let generation = if current.generation % 2 == 0 {
                current.generation.wrapping_add(2)
            } else {
                current.generation
            };
            Arc::new(BridgeEntryState {
                generation,
                provider: Some(Arc::clone(&provider)),
            })
        });
    }

    fn restore_provider(&self, provider: Arc<dyn Any + Send + Sync>) {
        self.state.rcu(|current| {
            let same_provider = current
                .provider
                .as_ref()
                .is_some_and(|current_provider| Arc::ptr_eq(current_provider, &provider));
            if current.generation % 2 == 0 && same_provider {
                return Arc::clone(current);
            }
            let generation = if current.generation % 2 == 0 {
                current.generation.wrapping_add(2)
            } else {
                current.generation.wrapping_add(1)
            };
            Arc::new(BridgeEntryState {
                generation,
                provider: Some(Arc::clone(&provider)),
            })
        });
    }

    fn snapshot_state(&self) -> BridgeEntrySnapshotState {
        let state = self.state.load();
        let generation = state.generation;
        let provider_installed = state.provider.is_some();
        let status = BridgeInterfaceStatus::from_installed_entry(generation, provider_installed);
        BridgeEntrySnapshotState {
            generation,
            provider_installed,
            status,
        }
    }
}

impl fmt::Debug for BridgeEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BridgeEntry")
            .field("interface_id", &self.interface_id)
            .field("state", &self.snapshot_state())
            .field("owner", &self.owner)
            .field("diagnostics", &self.diagnostics())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BridgeEntrySnapshotState {
    generation: u32,
    provider_installed: bool,
    status: BridgeInterfaceStatus,
}

#[derive(Clone, Debug)]
/// 注册表合并后的拓扑快照：接口 ID 到 slot 不再改变；每个 slot 的启停与 provider 可随生命周期发布。
pub struct FrozenBridgeTable {
    inner: Arc<FrozenBridgeTableInner>,
}

#[derive(Debug)]
struct FrozenBridgeTableInner {
    entries: Box<[BridgeEntry]>,
    slots_by_interface: HashMap<String, InterfaceSlot>,
}

impl FrozenBridgeTable {
    /// 仅用于已去重的注册表导出；遍历顺序决定 slot，调用方不得把未验证的重复 ID 直接送入。
    pub(crate) fn from_exports(
        exports: impl IntoIterator<Item = (PluginModuleId, String, InterfaceExport)>,
    ) -> Self {
        let exports = exports.into_iter();
        let (export_count, _) = exports.size_hint();
        let mut entries = Vec::with_capacity(export_count);
        let mut slots_by_interface = HashMap::with_capacity(export_count);
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

    /// Whether both handles retain the same immutable table allocation.
    ///
    /// Registration-replay generations capture bridge call scopes, so cache reuse is safe only
    /// when the caller is replaying through this exact frozen table, not just an equivalent table.
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
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

    /// 强句柄直接持有 provider；目录的强依赖检查必须先阻止目标停用，句柄本身不会自动撤销。
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

    /// 弱句柄保留固定 slot 与冻结表，可观察禁用和热更新；不存在的接口留为 Absent。
    pub fn resolve_weak<T>(&self) -> WeakBridge<T>
    where
        T: PluginInterface + ?Sized,
    {
        WeakBridge::<T>::new(self.clone(), self.resolve_slot(T::INTERFACE_ID))
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

    /// 停用清空 provider 后，按最终注册表同 owner 的导出恢复原 slot；报告用于生命周期诊断。
    pub(crate) fn restore_owner_exports_with_report<'a>(
        &self,
        owner: PluginModuleId,
        exports: impl IntoIterator<Item = (&'a str, &'a InterfaceExport)>,
    ) -> BridgeOwnerTransitionReport {
        let mut affected_slots = Vec::new();
        for (interface_id, export) in exports {
            let Some(slot) = self.resolve_slot(interface_id) else {
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

    /// 替换注册表提供的新对象沿旧 slot 发布；调用方必须保证接口 ID 与原 trait 类型一致。
    pub(crate) fn reload_owner_exports_with_report<'a>(
        &self,
        owner: PluginModuleId,
        exports: impl IntoIterator<Item = (&'a str, &'a InterfaceExport)>,
    ) -> BridgeOwnerTransitionReport {
        let mut affected_slots = Vec::new();
        for (interface_id, export) in exports {
            let Some(slot) = self.resolve_slot(interface_id) else {
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

    /// 公开的按 slot 热更新入口；T 的静态接口 ID 应与该 slot 的接口 ID 相同。
    // BUG: [CR-PLUGIN-BOUNDARY-0001] 这里只检查 slot 存在，未核对 T::INTERFACE_ID；错位替换会把 Enabled 槽变成类型下转失败的 NotEnabled 调用。
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
        for (_, entry) in entries {
            let state = entry.snapshot_state();
            summary.record_state(state.status, state.provider_installed, entry.diagnostics());
        }
        summary
    }

    // 在变更之后快照受影响 slot，向目录和编辑器报告同一 owner 的实际发布状态。
    fn owner_transition_report(
        &self,
        owner: PluginModuleId,
        mode: BridgeOwnerTransitionMode,
        affected_slots: Vec<InterfaceSlot>,
    ) -> BridgeOwnerTransitionReport {
        let mut snapshots = Vec::with_capacity(affected_slots.len());
        for slot in &affected_slots {
            if let Some(entry) = self.entry(*slot) {
                snapshots.push(self.snapshot_for_entry(slot.index(), entry));
            }
        }
        BridgeOwnerTransitionReport {
            owner,
            mode,
            affected_slots,
            snapshots,
        }
    }
}

impl BridgeInvocationTable for FrozenBridgeTable {
    fn resolve_interface_slot(&self, interface_id: &str) -> Option<InterfaceSlot> {
        self.resolve_slot(interface_id)
    }

    fn interface_status_at(&self, slot: InterfaceSlot) -> BridgeInterfaceStatus {
        self.entry(slot)
            .map_or(BridgeInterfaceStatus::Absent, BridgeEntry::status)
    }

    fn record_enabled_call(&self, slot: InterfaceSlot) {
        FrozenBridgeTable::record_enabled_call(self, slot);
    }

    fn record_not_enabled_call(&self, slot: InterfaceSlot) {
        FrozenBridgeTable::record_not_enabled_call(self, slot);
    }
}

#[cfg(test)]
#[path = "tests/table.rs"]
mod tests;

#[cfg(test)]
#[path = "table/tests/optimization_tests.rs"]
mod optimization_tests;
