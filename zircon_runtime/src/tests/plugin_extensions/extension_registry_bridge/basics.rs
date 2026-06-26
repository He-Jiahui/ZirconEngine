use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

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
