use crate::plugin::{PluginEventManifest, RuntimeExtensionRegistry};
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::CoreRuntime;
use crate::scene::ecs::{
    Res, ResMut, ResMutParam, ResParam, Resource, RuntimeSceneSystemContext, SystemRef, SystemStage,
};
use crate::scene::World;
use crate::scene::{create_default_level, module_descriptor};

#[derive(Debug, PartialEq, Eq)]
struct WeatherConfig(u32);

impl Resource for WeatherConfig {}

#[derive(Debug, PartialEq, Eq)]
struct WeatherObserved(Vec<u32>);

impl Resource for WeatherObserved {}

#[derive(Debug, PartialEq, Eq)]
struct WeatherChanged;

#[test]
fn plugin_resource_event_and_system_registrations_apply_to_world() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");
    let weather_set = registry
        .intern_system_set("weather.main")
        .expect("plugin system set id");

    registry
        .register_resource::<WeatherConfig>(owner, || WeatherConfig(7))
        .unwrap();
    registry
        .register_resource::<WeatherObserved>(owner, || WeatherObserved(Vec::new()))
        .unwrap();
    registry
        .register_event::<WeatherChanged>(
            owner,
            PluginEventManifest {
                id: "weather.events.changed".to_string(),
                display_name: "Weather Changed".to_string(),
                payload_schema: "weather.schemas.changed.v1".to_string(),
            },
        )
        .unwrap();
    registry
        .register_native_system::<(ResParam<WeatherConfig>, ResMutParam<WeatherObserved>), _>(
            owner,
            "weather.apply",
            SystemStage::Update,
            |(config, mut observed): (Res<'_, WeatherConfig>, ResMut<'_, WeatherObserved>)| {
                observed.0.push(config.0);
            },
        )
        .in_set(weather_set)
        .register()
        .unwrap();

    let ownership = registry.ownership_for(owner);
    assert_eq!(ownership.plugin_resources.len(), 2);
    assert_eq!(ownership.plugin_events.len(), 1);
    assert_eq!(ownership.plugin_event_catalogs.len(), 1);
    assert_eq!(ownership.plugin_systems.len(), 1);

    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(world.resource::<WeatherConfig>(), &WeatherConfig(7));
    assert_eq!(
        world.resource::<WeatherObserved>(),
        &WeatherObserved(vec![7])
    );
    assert!(world.events::<WeatherChanged>().is_some());
    assert!(registry.plugin_event_catalogs().iter().any(|catalog| {
        catalog.namespace == "weather.events"
            && catalog
                .events
                .iter()
                .any(|event| event.id == "weather.events.changed")
    }));
}

#[test]
fn plugin_system_constraints_order_registered_native_systems() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");
    let set = registry
        .intern_system_set("weather.main")
        .expect("plugin system set id");

    registry
        .register_resource::<WeatherObserved>(owner, || WeatherObserved(Vec::new()))
        .unwrap();
    registry
        .register_native_system::<ResMutParam<WeatherObserved>, _>(
            owner,
            "weather.second",
            SystemStage::Update,
            |mut observed: ResMut<'_, WeatherObserved>| observed.0.push(2),
        )
        .in_set(set)
        .with_order(-100)
        .after(SystemRef::System("weather.first".to_string()))
        .register()
        .unwrap();
    registry
        .register_native_system::<ResMutParam<WeatherObserved>, _>(
            owner,
            "weather.first",
            SystemStage::Update,
            |mut observed: ResMut<'_, WeatherObserved>| observed.0.push(1),
        )
        .in_set(set)
        .with_order(100)
        .register()
        .unwrap();

    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(
        world.resource::<WeatherObserved>(),
        &WeatherObserved(vec![1, 2])
    );
}

#[test]
fn plugin_system_lands_in_stage_plan() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");
    let set = registry
        .intern_system_set("weather.main")
        .expect("plugin system set id");
    registry
        .register_resource::<WeatherConfig>(owner, || WeatherConfig(7))
        .unwrap();
    registry
        .register_native_system::<ResParam<WeatherConfig>, _>(
            owner,
            "weather.stage-plan-reader",
            SystemStage::Update,
            |_: Res<'_, WeatherConfig>| {},
        )
        .in_set(set)
        .register()
        .unwrap();

    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();

    let graph = world
        .schedule()
        .native_system_conflict_graph_for_stage(SystemStage::Update);
    assert!(graph.nodes().iter().any(|node| {
        node.system_id() == "weather.stage-plan-reader" && node.stage() == SystemStage::Update
    }));
}

#[test]
fn plugin_system_access_joins_conflict_graph() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");
    registry
        .register_resource::<WeatherConfig>(owner, || WeatherConfig(7))
        .unwrap();
    registry
        .register_native_system::<ResParam<WeatherConfig>, _>(
            owner,
            "weather.read-config",
            SystemStage::Update,
            |_: Res<'_, WeatherConfig>| {},
        )
        .register()
        .unwrap();
    registry
        .register_native_system::<ResMutParam<WeatherConfig>, _>(
            owner,
            "weather.write-config",
            SystemStage::Update,
            |_: ResMut<'_, WeatherConfig>| {},
        )
        .register()
        .unwrap();

    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();

    let graph = world
        .schedule()
        .native_system_conflict_graph_for_stage(SystemStage::Update);
    assert!(graph.has_conflicts());
    assert!(graph.edges().iter().any(|edge| {
        let endpoints = (edge.left_system_id(), edge.right_system_id());
        endpoints == ("weather.read-config", "weather.write-config")
            || endpoints == ("weather.write-config", "weather.read-config")
    }));
}

#[test]
fn plugin_runtime_scene_system_registrations_apply_to_world() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));

    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");
    let weather_set = registry
        .intern_system_set("weather.runtime")
        .expect("plugin system set id");
    {
        let events = events.clone();
        registry
            .register_runtime_scene_system(
                owner,
                "weather.runtime-context",
                SystemStage::Update,
                move |context: RuntimeSceneSystemContext<'_>| {
                    context.level.with_world(|_| {
                        events
                            .lock()
                            .unwrap()
                            .push(format!("runtime-context={:.3}", context.delta_seconds));
                    });
                    Ok(())
                },
            )
            .in_set(weather_set)
            .register()
            .unwrap();
    }

    let ownership = registry.ownership_for(owner);
    assert_eq!(ownership.plugin_runtime_systems.len(), 1);

    level
        .with_world_mut(|world| registry.apply_to_world(world))
        .unwrap();
    let advance = runtime.advance_time_by(std::time::Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec!["runtime-context=0.025".to_string()]
    );
}
