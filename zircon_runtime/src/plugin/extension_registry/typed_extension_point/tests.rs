use super::TypedExtensionPoint;
use crate::plugin::PluginModuleId;

#[test]
fn frozen_table_dense_lookup_matches_registration() {
    let owner = PluginModuleId::from_raw(7);
    let mut point = TypedExtensionPoint::<String, u32>::default();
    let first = point
        .register(owner, "weather.first".to_string(), 11)
        .expect("first extension");
    let second = point
        .register(owner, "weather.second".to_string(), 29)
        .expect("second extension");

    let frozen = point.finalize();

    assert_eq!(first.raw(), 0);
    assert_eq!(second.raw(), 1);
    assert_eq!(frozen.resolve(&"weather.first".to_string()), Some(first));
    assert_eq!(frozen.resolve(&"weather.second".to_string()), Some(second));
    assert_eq!(frozen.get(first), Some(&11));
    assert_eq!(frozen.get(second), Some(&29));
    assert_eq!(frozen.owner_for_slot(second), Some(owner));
}

#[test]
fn duplicate_extension_key_rejected() {
    let owner = PluginModuleId::from_raw(3);
    let mut point = TypedExtensionPoint::<String, u32>::default();
    let first = point
        .register(owner, "weather.shared".to_string(), 1)
        .expect("first extension");

    let duplicate = point
        .register(owner, "weather.shared".to_string(), 2)
        .expect_err("duplicate extension key must be rejected");

    assert_eq!(duplicate, first);
    assert_eq!(point.values(), &[1]);
}

#[test]
fn owner_revocation_preserves_survivor_slots_and_retires_removed_slots() {
    let weather = PluginModuleId::from_raw(3);
    let storm = PluginModuleId::from_raw(5);
    let mut point = TypedExtensionPoint::<String, u32>::default();
    let weather_slot = point
        .register(weather, "weather.clouds".to_string(), 11)
        .expect("weather extension");
    let storm_slot = point
        .register(storm, "storm.lightning".to_string(), 29)
        .expect("storm extension");
    point.freeze();
    assert!(point.is_frozen());

    assert_eq!(point.remove_owned_by(weather), vec![weather_slot]);
    assert!(!point.is_frozen());
    assert_eq!(
        point.resolve(&"storm.lightning".to_string()),
        Some(storm_slot)
    );
    assert_eq!(point.get(storm_slot), Some(&29));
    assert_eq!(point.owner_for_slot(storm_slot), Some(storm));
    assert_eq!(point.get(weather_slot), None);
    assert_eq!(point.key_for_slot(weather_slot), None);
    assert_eq!(point.owner_for_slot(weather_slot), None);

    let reloaded_weather_slot = point
        .register(weather, "weather.clouds".to_string(), 41)
        .expect("reloaded weather extension");
    assert!(reloaded_weather_slot.raw() > storm_slot.raw());
    assert_eq!(
        point.resolve(&"weather.clouds".to_string()),
        Some(reloaded_weather_slot)
    );
    assert_eq!(point.get(reloaded_weather_slot), Some(&41));
    assert_eq!(point.get(weather_slot), None);

    let frozen = point.finalize();
    assert_eq!(frozen.get(weather_slot), None);
    assert_eq!(frozen.get(storm_slot), Some(&29));
    assert_eq!(frozen.get(reloaded_weather_slot), Some(&41));
}

#[test]
fn sorting_dense_rows_preserves_logical_slots() {
    let owner = PluginModuleId::from_raw(9);
    let mut point = TypedExtensionPoint::<String, u32>::default();
    let high_slot = point
        .register(owner, "weather.high".to_string(), 29)
        .expect("high extension");
    let low_slot = point
        .register(owner, "weather.low".to_string(), 11)
        .expect("low extension");

    point.sort_by_values(u32::cmp);

    assert_eq!(point.values(), &[11, 29]);
    assert_eq!(point.resolve(&"weather.high".to_string()), Some(high_slot));
    assert_eq!(point.resolve(&"weather.low".to_string()), Some(low_slot));
    assert_eq!(point.get(high_slot), Some(&29));
    assert_eq!(point.get(low_slot), Some(&11));
}
