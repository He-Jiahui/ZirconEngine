use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::core::framework::bridge::{BridgeError, PluginInterface};
use crate::plugin::{
    BridgeInterfaceSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionMode,
    BridgeOwnerTransitionReport, CapabilityView, InterfaceSlot, PluginFinishContext,
    PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
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

#[test]
fn duplicate_interface_export_rejected() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 21 }),
        )
        .unwrap();

    let error = registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 24 }),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RuntimeExtensionRegistryError::DuplicatePluginInterface("weather.query.v1".to_string())
    );
}

#[test]
fn weak_call_returns_absent_when_target_not_installed() {
    let registry = RuntimeExtensionRegistry::default();
    let table = registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::Absent)
    );
}

#[test]
fn strong_bridge_calls_exported_interface_directly() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 19 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();

    let bridge = table
        .resolve_strong::<dyn WeatherQueryInterface>()
        .expect("strong bridge");

    assert_eq!(bridge.sample_temperature(), 19);
}

#[test]
fn generation_parity_encodes_enabled_state() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 3 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let entry = table.entry(slot).unwrap();

    assert_eq!(entry.generation(), 0);
    assert!(entry.is_enabled());

    table.set_enabled(slot, false).unwrap();
    let disabled_generation = entry.generation();
    assert_eq!(disabled_generation % 2, 1);
    assert!(!entry.is_enabled());

    table.set_enabled(slot, false).unwrap();
    assert_eq!(entry.generation(), disabled_generation);

    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 4 }),
        )
        .unwrap();
    assert_eq!(entry.generation() % 2, 1);
    assert!(!entry.is_enabled());

    table.set_enabled(slot, true).unwrap();
    assert_eq!(entry.generation() % 2, 0);
    assert!(entry.is_enabled());

    table.set_enabled(slot, true).unwrap();
    assert_eq!(entry.generation() % 2, 0);
}

#[test]
fn weak_bridge_reconnects_after_generation_flip() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 7 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(bridge.call(|provider| provider.sample_temperature()), Ok(7));
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    table.set_enabled(slot, false).unwrap();
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );
    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 11 }),
        )
        .unwrap();
    table.set_enabled(slot, true).unwrap();

    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(11)
    );
}

#[test]
fn hot_reload_swaps_provider_without_caller_rewiring() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 18 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(18)
    );
    let original_generation = table.entry(slot).unwrap().generation();

    table
        .reload_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 22 }),
        )
        .unwrap();

    assert_eq!(
        table.entry(slot).unwrap().generation(),
        original_generation + 2
    );
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(22)
    );
}

#[test]
fn bridge_table_reports_interface_status_for_diagnostics() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 6 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();

    assert_eq!(
        table.interface_status(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID),
        BridgeInterfaceStatus::Enabled
    );
    assert_eq!(
        table.interface_status(<dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID),
        BridgeInterfaceStatus::Absent
    );

    table.set_enabled(slot, false).unwrap();

    assert_eq!(
        table.interface_status(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID),
        BridgeInterfaceStatus::Disabled
    );
}

#[test]
fn bridge_table_snapshots_interfaces_for_diagnostics_matrix() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let ocean_owner = registry.intern_plugin_module("ocean.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 10 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn ClimateQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 20 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn OceanQueryInterface>(
            ocean_owner,
            Arc::new(WeatherQueryProvider { temperature: 30 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let weather_slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let climate_slot = table
        .resolve_slot(<dyn ClimateQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let ocean_slot = table
        .resolve_slot(<dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let weather_bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    weather_bridge
        .call(|provider| provider.sample_temperature())
        .unwrap();
    table.set_enabled(climate_slot, false).unwrap();

    let snapshots = table.interface_snapshots();

    assert_eq!(snapshots.len(), 3);
    let observed_enabled_calls = debug_bridge_counter_value(1);
    assert_bridge_snapshot(
        &snapshots[0],
        weather_slot,
        <dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID,
        weather_owner,
        0,
        true,
        BridgeInterfaceStatus::Enabled,
        observed_enabled_calls,
        0,
    );
    assert_bridge_snapshot(
        &snapshots[1],
        climate_slot,
        <dyn ClimateQueryInterface as PluginInterface>::INTERFACE_ID,
        weather_owner,
        1,
        true,
        BridgeInterfaceStatus::Disabled,
        0,
        0,
    );
    assert_bridge_snapshot(
        &snapshots[2],
        ocean_slot,
        <dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID,
        ocean_owner,
        0,
        true,
        BridgeInterfaceStatus::Enabled,
        0,
        0,
    );
}

#[test]
fn bridge_table_filters_interface_snapshots_by_owner() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let ocean_owner = registry.intern_plugin_module("ocean.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 1 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn ClimateQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 2 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn OceanQueryInterface>(
            ocean_owner,
            Arc::new(WeatherQueryProvider { temperature: 3 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();

    let weather_snapshots = table.interface_snapshots_owned_by(weather_owner);
    let ocean_snapshots = table.interface_snapshots_owned_by(ocean_owner);

    assert_eq!(
        snapshot_interface_ids(&weather_snapshots),
        vec!["weather.query.v1", "weather.climate.v1"]
    );
    assert!(weather_snapshots
        .iter()
        .all(|snapshot| snapshot.owner == weather_owner));
    assert_eq!(
        snapshot_interface_ids(&ocean_snapshots),
        vec!["ocean.query.v1"]
    );
    assert!(ocean_snapshots
        .iter()
        .all(|snapshot| snapshot.owner == ocean_owner));
}

#[test]
fn bridge_table_resolves_single_interface_snapshot() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 1 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();

    let by_slot = table.interface_snapshot(slot).unwrap();
    let by_id = table
        .interface_snapshot_by_id(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();

    assert_eq!(by_slot, by_id);
    assert_eq!(by_slot.slot, slot);
    assert_eq!(by_slot.interface_id, "weather.query.v1");
    assert_eq!(by_slot.owner, owner);
    assert!(by_slot.provider_installed);
    assert_eq!(table.interface_snapshot_by_id("missing.interface.v1"), None);
}

#[test]
fn bridge_table_flips_all_interfaces_owned_by_plugin_module() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let ocean_owner = registry.intern_plugin_module("ocean.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 7 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn ClimateQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 9 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn OceanQueryInterface>(
            ocean_owner,
            Arc::new(WeatherQueryProvider { temperature: 3 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let weather_bridge = table.resolve_weak::<dyn WeatherQueryInterface>();
    let climate_bridge = table.resolve_weak::<dyn ClimateQueryInterface>();
    let ocean_bridge = table.resolve_weak::<dyn OceanQueryInterface>();

    let disabled_slots = table.set_owner_enabled(weather_owner, false);

    assert_eq!(disabled_slots.len(), 2);
    assert_eq!(
        weather_bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );
    assert_eq!(
        climate_bridge.call(|provider| provider.sample_humidity()),
        Err(BridgeError::NotEnabled)
    );
    assert_eq!(
        ocean_bridge.call(|provider| provider.sample_wave_height()),
        Ok(5)
    );

    let enabled_slots = table.set_owner_enabled(weather_owner, true);

    assert_eq!(enabled_slots, disabled_slots);
    assert_eq!(
        weather_bridge.call(|provider| provider.sample_temperature()),
        Ok(7)
    );
    assert_eq!(
        climate_bridge.call(|provider| provider.sample_humidity()),
        Ok(69)
    );
}

#[test]
fn bridge_table_reports_owner_enabled_transition() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let ocean_owner = registry.intern_plugin_module("ocean.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 4 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn ClimateQueryInterface>(
            weather_owner,
            Arc::new(WeatherQueryProvider { temperature: 5 }),
        )
        .unwrap();
    registry
        .export_interface::<dyn OceanQueryInterface>(
            ocean_owner,
            Arc::new(WeatherQueryProvider { temperature: 6 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();

    let report = table.set_owner_enabled_with_report(weather_owner, false);

    assert_owner_report(
        &report,
        weather_owner,
        BridgeOwnerTransitionMode::Disable,
        vec!["weather.query.v1", "weather.climate.v1"],
        true,
        BridgeInterfaceStatus::Disabled,
    );
    assert_eq!(
        report.diagnostic(),
        "bridge.owner_transition: owner module slot 0 Disable affected 2 interface(s): [`weather.query.v1`@slot0 generation=1 provider_installed=true status=Disabled, `weather.climate.v1`@slot1 generation=1 provider_installed=true status=Disabled]"
    );

    let activate_report = table.activate_owner_with_report(weather_owner);

    assert_owner_report(
        &activate_report,
        weather_owner,
        BridgeOwnerTransitionMode::Activate,
        vec!["weather.query.v1", "weather.climate.v1"],
        true,
        BridgeInterfaceStatus::Enabled,
    );
    assert_eq!(
        activate_report.diagnostic(),
        "bridge.owner_transition: owner module slot 0 Activate affected 2 interface(s): [`weather.query.v1`@slot0 generation=2 provider_installed=true status=Enabled, `weather.climate.v1`@slot1 generation=2 provider_installed=true status=Enabled]"
    );
    assert_eq!(
        table.interface_status(<dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID),
        BridgeInterfaceStatus::Enabled
    );
}

#[test]
fn bridge_table_deactivates_owner_by_disabling_and_clearing_providers() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 8 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(bridge.call(|provider| provider.sample_temperature()), Ok(8));

    let affected = table.deactivate_owner(owner);

    assert_eq!(affected, vec![slot]);
    let disabled_generation = table.entry(slot).unwrap().generation();
    assert_eq!(disabled_generation % 2, 1);
    assert!(!table.entry(slot).unwrap().is_enabled());
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );

    table.activate_owner(owner);
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );

    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 12 }),
        )
        .unwrap();
    assert_eq!(table.entry(slot).unwrap().generation(), disabled_generation);
    table.activate_owner(owner);

    assert_eq!(
        table.entry(slot).unwrap().generation(),
        disabled_generation + 1
    );
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(12)
    );
}

#[test]
fn bridge_table_reports_owner_deactivation_transition() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 8 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    let report = table.deactivate_owner_with_report(owner);

    assert_owner_report(
        &report,
        owner,
        BridgeOwnerTransitionMode::Deactivate,
        vec!["weather.query.v1"],
        false,
        BridgeInterfaceStatus::Disabled,
    );
    assert_eq!(
        report.diagnostic(),
        "bridge.owner_transition: owner module slot 0 Deactivate affected 1 interface(s): [`weather.query.v1`@slot0 generation=1 provider_installed=false status=Disabled]"
    );
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );
}

#[test]
fn pin_guard_amortizes_weak_bridge_resolution() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(CountingWeatherProvider {
                calls: calls.clone(),
            }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    let guard = bridge.pin().expect("weak bridge guard");
    assert_eq!(guard.sample_temperature(), 5);
    assert_eq!(guard.sample_temperature(), 5);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
#[cfg(debug_assertions)]
fn weak_bridge_records_debug_diagnostics() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 13 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(table.diagnostics(slot).unwrap().enabled_calls, 0);
    assert_eq!(table.diagnostics(slot).unwrap().not_enabled_calls, 0);

    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(13)
    );
    table.set_enabled(slot, false).unwrap();
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );

    let diagnostics = table.diagnostics(slot).unwrap();
    assert_eq!(diagnostics.enabled_calls, 1);
    assert_eq!(diagnostics.not_enabled_calls, 1);
}

#[test]
fn finish_context_resolves_strong_and_weak_interfaces() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 17 }),
        )
        .unwrap();
    let capabilities = CapabilityView::default();
    let context = PluginFinishContext::new(&mut registry, &capabilities);

    let strong = context
        .resolve_strong::<dyn WeatherQueryInterface>()
        .expect("strong bridge");
    let weak = context.resolve_weak::<dyn WeatherQueryInterface>();

    assert_eq!(strong.sample_temperature(), 17);
    assert_eq!(weak.call(|provider| provider.sample_temperature()), Ok(17));
}

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

#[derive(Debug)]
struct CountingWeatherProvider {
    calls: Arc<AtomicUsize>,
}

impl WeatherQueryInterface for CountingWeatherProvider {
    fn sample_temperature(&self) -> i32 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        5
    }
}
