use super::*;

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
fn bridge_table_summarizes_diagnostics_for_matrix() {
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
    let climate_slot = table
        .resolve_slot(<dyn ClimateQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let ocean_slot = table
        .resolve_slot(<dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let weather_bridge = table.resolve_weak::<dyn WeatherQueryInterface>();
    let climate_bridge = table.resolve_weak::<dyn ClimateQueryInterface>();
    let ocean_bridge = table.resolve_weak::<dyn OceanQueryInterface>();

    assert_eq!(
        weather_bridge.call(|provider| provider.sample_temperature()),
        Ok(10)
    );
    table.set_enabled(climate_slot, false).unwrap();
    assert_eq!(
        climate_bridge.call(|provider| provider.sample_humidity()),
        Err(BridgeError::NotEnabled)
    );
    assert_eq!(table.deactivate_owner(ocean_owner), vec![ocean_slot]);
    assert_eq!(
        ocean_bridge.call(|provider| provider.sample_wave_height()),
        Err(BridgeError::NotEnabled)
    );

    let summary = table.diagnostics_summary();

    assert_eq!(
        summary,
        BridgeTableDiagnosticsSummary {
            total_interfaces: 3,
            enabled_interfaces: 1,
            disabled_interfaces: 2,
            installed_providers: 2,
            missing_providers: 1,
            enabled_calls: debug_bridge_counter_value(1),
            not_enabled_calls: debug_bridge_counter_value(2),
        }
    );
    assert_eq!(
        summary.diagnostic(),
        format!(
            "bridge.table_summary: total=3 enabled=1 disabled=2 providers_installed=2 providers_missing=1 enabled_calls={} not_enabled_calls={}",
            debug_bridge_counter_value(1),
            debug_bridge_counter_value(2)
        )
    );
    assert_eq!(
        table.diagnostics_summary_owned_by(weather_owner),
        BridgeTableDiagnosticsSummary {
            total_interfaces: 2,
            enabled_interfaces: 1,
            disabled_interfaces: 1,
            installed_providers: 2,
            missing_providers: 0,
            enabled_calls: debug_bridge_counter_value(1),
            not_enabled_calls: debug_bridge_counter_value(1),
        }
    );
    assert_eq!(
        table.diagnostics_summary_owned_by(ocean_owner),
        BridgeTableDiagnosticsSummary {
            total_interfaces: 1,
            enabled_interfaces: 0,
            disabled_interfaces: 1,
            installed_providers: 0,
            missing_providers: 1,
            enabled_calls: 0,
            not_enabled_calls: debug_bridge_counter_value(1),
        }
    );
}

#[test]
fn bridge_diagnostics_matrix_projects_editor_rows() {
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
        .export_interface::<dyn OceanQueryInterface>(
            ocean_owner,
            Arc::new(WeatherQueryProvider { temperature: 30 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let weather_slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let ocean_slot = table
        .resolve_slot(<dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let weather_bridge = table.resolve_weak::<dyn WeatherQueryInterface>();
    let ocean_bridge = table.resolve_weak::<dyn OceanQueryInterface>();

    assert_eq!(
        weather_bridge.call(|provider| provider.sample_temperature()),
        Ok(10)
    );
    assert_eq!(table.deactivate_owner(ocean_owner), vec![ocean_slot]);
    assert_eq!(
        ocean_bridge.call(|provider| provider.sample_wave_height()),
        Err(BridgeError::NotEnabled)
    );

    let matrix = table.diagnostics_matrix();

    assert_eq!(
        matrix.summary,
        BridgeTableDiagnosticsSummary {
            total_interfaces: 2,
            enabled_interfaces: 1,
            disabled_interfaces: 1,
            installed_providers: 1,
            missing_providers: 1,
            enabled_calls: debug_bridge_counter_value(1),
            not_enabled_calls: debug_bridge_counter_value(1),
        }
    );
    assert_eq!(matrix.rows.len(), 2);
    assert_bridge_snapshot(
        &matrix.rows[0],
        weather_slot,
        <dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID,
        weather_owner,
        0,
        true,
        BridgeInterfaceStatus::Enabled,
        debug_bridge_counter_value(1),
        0,
    );
    assert_bridge_snapshot(
        &matrix.rows[1],
        ocean_slot,
        <dyn OceanQueryInterface as PluginInterface>::INTERFACE_ID,
        ocean_owner,
        1,
        false,
        BridgeInterfaceStatus::Disabled,
        0,
        debug_bridge_counter_value(1),
    );
    assert_eq!(
        matrix.diagnostic_lines(),
        vec![
            format!(
                "bridge.table_summary: total=2 enabled=1 disabled=1 providers_installed=1 providers_missing=1 enabled_calls={} not_enabled_calls={}",
                debug_bridge_counter_value(1),
                debug_bridge_counter_value(1)
            ),
            format!(
                "bridge.interface: slot={} interface=`weather.query.v1` owner_module_slot={} generation=0 provider_installed=true status=Enabled enabled_calls={} not_enabled_calls=0",
                weather_slot.raw(),
                weather_owner.raw(),
                debug_bridge_counter_value(1)
            ),
            format!(
                "bridge.interface: slot={} interface=`ocean.query.v1` owner_module_slot={} generation=1 provider_installed=false status=Disabled enabled_calls=0 not_enabled_calls={}",
                ocean_slot.raw(),
                ocean_owner.raw(),
                debug_bridge_counter_value(1)
            ),
        ]
    );
    assert_eq!(
        table.diagnostics_matrix_owned_by(weather_owner),
        BridgeDiagnosticsMatrix {
            summary: BridgeTableDiagnosticsSummary {
                total_interfaces: 1,
                enabled_interfaces: 1,
                disabled_interfaces: 0,
                installed_providers: 1,
                missing_providers: 0,
                enabled_calls: debug_bridge_counter_value(1),
                not_enabled_calls: 0,
            },
            rows: vec![matrix.rows[0].clone()],
        }
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
