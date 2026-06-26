use super::*;

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
fn bridge_table_reloads_owner_exports_with_report() {
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
    let original_generation = table.entry(slot).unwrap().generation();

    let mut replacement_registry = RuntimeExtensionRegistry::default();
    let replacement_owner = replacement_registry
        .intern_plugin_module("weather.runtime")
        .unwrap();
    replacement_registry
        .export_interface::<dyn WeatherQueryInterface>(
            replacement_owner,
            Arc::new(WeatherQueryProvider { temperature: 24 }),
        )
        .unwrap();

    let report = table.reload_owner_exports_with_report(
        owner,
        replacement_registry.interface_exports_owned_by(replacement_owner),
    );

    assert_owner_report(
        &report,
        owner,
        BridgeOwnerTransitionMode::Reload,
        vec!["weather.query.v1"],
        true,
        BridgeInterfaceStatus::Enabled,
    );
    assert_eq!(
        table.entry(slot).unwrap().generation(),
        original_generation + 2
    );
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(24)
    );
    assert!(report.diagnostic().contains("Reload affected 1 interface"));
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
