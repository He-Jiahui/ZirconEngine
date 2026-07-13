use std::fmt;

use crate::plugin::extension_registry::PluginModuleId;

use crate::core::framework::bridge::{
    BridgeDiagnosticsSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionMode, InterfaceSlot,
};

impl fmt::Debug for super::InterfaceExport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InterfaceExport")
            .field("interface_id", &self.interface_id)
            .finish_non_exhaustive()
    }
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

impl BridgeInterfaceSnapshot {
    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.interface: slot={} interface=`{}` owner_module_slot={} generation={} provider_installed={} status={:?} enabled_calls={} not_enabled_calls={}",
            self.slot.raw(),
            self.interface_id,
            self.owner.raw(),
            self.generation,
            self.provider_installed,
            self.status,
            self.diagnostics.enabled_calls,
            self.diagnostics.not_enabled_calls
        )
    }
}

/// Aggregate bridge-table state for lifecycle logs and editor diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BridgeTableDiagnosticsSummary {
    pub total_interfaces: usize,
    pub enabled_interfaces: usize,
    pub disabled_interfaces: usize,
    pub installed_providers: usize,
    pub missing_providers: usize,
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

impl BridgeTableDiagnosticsSummary {
    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.table_summary: total={} enabled={} disabled={} providers_installed={} providers_missing={} enabled_calls={} not_enabled_calls={}",
            self.total_interfaces,
            self.enabled_interfaces,
            self.disabled_interfaces,
            self.installed_providers,
            self.missing_providers,
            self.enabled_calls,
            self.not_enabled_calls
        )
    }

    fn from_snapshots<'snapshot>(
        snapshots: impl IntoIterator<Item = &'snapshot BridgeInterfaceSnapshot>,
    ) -> Self {
        let mut summary = Self::default();
        for snapshot in snapshots {
            summary.record_snapshot(snapshot);
        }
        summary
    }

    pub(super) fn record_snapshot(&mut self, snapshot: &BridgeInterfaceSnapshot) {
        self.total_interfaces += 1;
        match snapshot.status {
            BridgeInterfaceStatus::Enabled => self.enabled_interfaces += 1,
            BridgeInterfaceStatus::Disabled => self.disabled_interfaces += 1,
            BridgeInterfaceStatus::Absent => {}
        }
        if snapshot.provider_installed {
            self.installed_providers += 1;
        } else {
            self.missing_providers += 1;
        }
        self.enabled_calls += snapshot.diagnostics.enabled_calls;
        self.not_enabled_calls += snapshot.diagnostics.not_enabled_calls;
    }
}

/// Editor-facing bridge diagnostics matrix: one summary row plus deterministic interface rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeDiagnosticsMatrix {
    pub summary: BridgeTableDiagnosticsSummary,
    pub rows: Vec<BridgeInterfaceSnapshot>,
}

impl BridgeDiagnosticsMatrix {
    pub(super) fn from_rows(rows: Vec<BridgeInterfaceSnapshot>) -> Self {
        let summary = BridgeTableDiagnosticsSummary::from_snapshots(&rows);
        Self { summary, rows }
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.rows.len() + 1);
        lines.push(self.summary.diagnostic());
        lines.extend(self.rows.iter().map(BridgeInterfaceSnapshot::diagnostic));
        lines
    }
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
