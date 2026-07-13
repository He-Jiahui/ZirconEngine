use std::sync::Arc;

use crate::core::framework::bridge::{
    BridgeDiagnosticsSnapshot, BridgeError, BridgeInterfaceStatus, BridgeOwnerTransitionMode,
    InterfaceSlot, PluginInterface,
};
use crate::plugin::{
    BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot, BridgeOwnerTransitionReport,
    BridgeTableDiagnosticsSummary, PluginModuleId, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, WeakBridge,
};

trait WeatherQueryInterface: Send + Sync {
    fn sample_temperature(&self) -> i32;
}

impl PluginInterface for dyn WeatherQueryInterface {
    const INTERFACE_ID: &'static str = "weather.query.v1";
}

trait ClimateQueryInterface: Send + Sync {
    fn sample_humidity(&self) -> i32;
}

impl PluginInterface for dyn ClimateQueryInterface {
    const INTERFACE_ID: &'static str = "weather.climate.v1";
}

trait OceanQueryInterface: Send + Sync {
    fn sample_wave_height(&self) -> i32;
}

impl PluginInterface for dyn OceanQueryInterface {
    const INTERFACE_ID: &'static str = "ocean.query.v1";
}

#[derive(Debug)]
struct WeatherQueryProvider {
    temperature: i32,
}

impl WeatherQueryInterface for WeatherQueryProvider {
    fn sample_temperature(&self) -> i32 {
        self.temperature
    }
}

impl ClimateQueryInterface for WeatherQueryProvider {
    fn sample_humidity(&self) -> i32 {
        self.temperature + 60
    }
}

impl OceanQueryInterface for WeatherQueryProvider {
    fn sample_wave_height(&self) -> i32 {
        self.temperature + 2
    }
}

#[path = "extension_registry_bridge/basics.rs"]
mod basics;
#[path = "extension_registry_bridge/diagnostics.rs"]
mod diagnostics;
#[path = "extension_registry_bridge/lifecycle.rs"]
mod lifecycle;

fn assert_bridge_snapshot(
    snapshot: &BridgeInterfaceSnapshot,
    slot: InterfaceSlot,
    interface_id: &str,
    owner: PluginModuleId,
    generation: u32,
    provider_installed: bool,
    status: BridgeInterfaceStatus,
    enabled_calls: u64,
    not_enabled_calls: u64,
) {
    assert_eq!(snapshot.slot, slot);
    assert_eq!(snapshot.interface_id, interface_id);
    assert_eq!(snapshot.owner, owner);
    assert_eq!(snapshot.generation, generation);
    assert_eq!(snapshot.provider_installed, provider_installed);
    assert_eq!(snapshot.status, status);
    assert_eq!(snapshot.diagnostics.enabled_calls, enabled_calls);
    assert_eq!(snapshot.diagnostics.not_enabled_calls, not_enabled_calls);
}

fn snapshot_interface_ids(snapshots: &[BridgeInterfaceSnapshot]) -> Vec<&str> {
    snapshots
        .iter()
        .map(|snapshot| snapshot.interface_id.as_str())
        .collect()
}

fn assert_owner_report(
    report: &BridgeOwnerTransitionReport,
    owner: PluginModuleId,
    mode: BridgeOwnerTransitionMode,
    interface_ids: Vec<&str>,
    provider_installed: bool,
    status: BridgeInterfaceStatus,
) {
    assert_eq!(report.owner, owner);
    assert_eq!(report.mode, mode);
    assert_eq!(report.affected_slots.len(), interface_ids.len());
    assert_eq!(snapshot_interface_ids(&report.snapshots), interface_ids);
    assert!(report
        .snapshots
        .iter()
        .all(|snapshot| snapshot.owner == owner
            && snapshot.provider_installed == provider_installed
            && snapshot.status == status));
    assert_eq!(
        report.affected_slots,
        report
            .snapshots
            .iter()
            .map(|snapshot| snapshot.slot)
            .collect::<Vec<_>>()
    );
}

#[cfg(debug_assertions)]
fn debug_bridge_counter_value(value: u64) -> u64 {
    value
}

#[cfg(not(debug_assertions))]
fn debug_bridge_counter_value(_: u64) -> u64 {
    0
}
