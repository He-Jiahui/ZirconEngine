use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
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
            || |mut counter: ResMut<'_, InstalledCounter>| counter.0 += 1,
        )
        .register()
        .expect("valid native scene system");

    for module_name in [
        foundation::FOUNDATION_MODULE_NAME,
        asset::ASSET_MODULE_NAME,
        SCENE_MODULE_NAME,
    ] {
        runtime.activate_module(module_name).unwrap();
    }

    let core = runtime.handle();
    let plan = registry
        .world_runtime_extension_plan()
        .expect("build world extension plan");
    scene::install_world_runtime_extension_plan(&core, plan).expect("install world extensions");

    let level = scene::create_default_level(&core).unwrap();
    level.tick(&core, runtime.tick_time(4)).unwrap();
    level.with_world(|world| {
        assert_eq!(world.resource::<InstalledCounter>(), &InstalledCounter(8));
        assert!(world.events::<InstalledEvent>().is_some());
    });
}
