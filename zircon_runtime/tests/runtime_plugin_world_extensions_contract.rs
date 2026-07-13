use zircon_runtime::core::CoreRuntime;
use zircon_runtime::plugin::{PluginEventManifest, RuntimeExtensionRegistry};
use zircon_runtime::scene::ecs::{ResMut, ResMutParam, Resource, SystemStage};
use zircon_runtime::{asset, foundation, scene};

#[derive(Debug, PartialEq, Eq)]
struct InstalledCounter(u32);

impl Resource for InstalledCounter {}

#[derive(Debug, PartialEq, Eq)]
struct InstalledEvent;

#[test]
fn installed_world_extensions_reach_new_levels() {
    let runtime = CoreRuntime::new();
    for descriptor in [
        foundation::module_descriptor(),
        asset::module_descriptor(),
        scene::module_descriptor(),
    ] {
        runtime.register_module(descriptor).unwrap();
    }

    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("valid plugin module owner");
    registry
        .register_resource(owner, || InstalledCounter(7))
        .expect("valid resource");
    registry
        .register_event::<InstalledEvent>(
            owner,
            PluginEventManifest {
                id: "weather.events.installed".to_string(),
                display_name: "Installed".to_string(),
                payload_schema: "weather.installed.v1".to_string(),
            },
        )
        .expect("valid event");
    registry
        .register_native_system::<ResMutParam<InstalledCounter>, _>(
            owner,
            "weather.increment",
            SystemStage::Update,
            |mut counter: ResMut<'_, InstalledCounter>| counter.0 += 1,
        )
        .register()
        .expect("valid native scene system");

    runtime
        .install_world_runtime_extensions(&registry)
        .expect("install world extensions");
    for module_name in [
        foundation::FOUNDATION_MODULE_NAME,
        asset::ASSET_MODULE_NAME,
        scene::SCENE_MODULE_NAME,
    ] {
        runtime.activate_module(module_name).unwrap();
    }

    let level = scene::create_default_level(&runtime.handle()).unwrap();
    level.with_world_mut(|world| {
        world.run_native_scene_systems_for_stage(SystemStage::Update);
        assert_eq!(world.resource::<InstalledCounter>(), &InstalledCounter(8));
        assert!(world.events::<InstalledEvent>().is_some());
    });
}
