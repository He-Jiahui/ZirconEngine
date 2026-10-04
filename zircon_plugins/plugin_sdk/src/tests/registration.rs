use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use serde_json::Value;
use zircon_runtime::asset::assets::SceneComponentAssetRecord;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::components::NodeRecord;
use zircon_runtime::scene::world::{SceneComponentSerializer, SceneProjectError, World};

const MODULE_OWNER: &str = "sdk_registration.runtime";
const MODULE_NAME: &str = "SdkRegistrationRuntimeModule";
const SYSTEM_SET: &str = "sdk_registration.update";
const SYSTEM_ID: &str = "sdk_registration.runtime.tick";
const DEFAULT_DOMAIN_SYSTEM_ID: &str = "sdk_registration.runtime.virtual-default";
const OPTION_ID: &str = "sdk_registration.option";
const CATALOG_NAMESPACE: &str = "sdk_registration.events";
const CATALOG_SEED_EVENT_ID: &str = "sdk_registration.events.seed";
const CATALOG_SEED_EVENT_SCHEMA: &str = "sdk_registration.seed.v1";
const EVENT_ID: &str = "sdk_registration.events.runtime_event";
const EVENT_SCHEMA: &str = "sdk_registration.runtime_event.v1";
const WORLD_TRANSFORM_SYSTEM: &str = "zircon.scene.world_transform";

#[derive(Clone, Debug)]
struct SdkRegistrationEvent;

#[derive(Clone, Debug, Default)]
struct SdkRegistrationResource;

impl Resource for SdkRegistrationResource {}

trait SdkImportedBridge: Send + Sync {}

impl PluginInterface for dyn SdkImportedBridge {
    const INTERFACE_ID: &'static str = "sdk.registration.imported.v1";
}

fn sdk_scene_codec_capture(
    _project: &ProjectManager,
    _world: &World,
    _record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    Ok(None)
}

fn sdk_scene_codec_instantiate(
    _project: &ProjectManager,
    _row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(None)
}

#[test]
fn runtime_registration_builder_hides_module_owner_sequence() {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_module(zircon_runtime::core::ModuleDescriptor::new(
            MODULE_NAME,
            "SDK registration builder test module",
        ))
        .expect("descriptor registered by runtime plugin report");
    let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
        .module(MODULE_OWNER)
        .expect("module registered");

    assert_eq!(module.module_name(), MODULE_OWNER);
    let module_owner = module.owner();
    let imported = module
        .import_interface::<dyn SdkImportedBridge>()
        .expect("interface import registered through SDK");

    module
        .component(ComponentTypeDescriptor::new(
            "sdk_registration.weather",
            "sdk_registration",
            "SDK Weather",
        ))
        .expect("component registered through SDK");
    module
        .scene_component_codec(SceneComponentSerializer {
            type_id: "sdk_registration.weather",
            schema_id: "sdk_registration.weather.v1",
            schema_version: 1,
            provider_id: "sdk_registration",
            capture: sdk_scene_codec_capture,
            instantiate: sdk_scene_codec_instantiate,
        })
        .expect("scene codec registered through SDK");

    module
        .resource(SdkRegistrationResource::default)
        .expect("runtime resource registered");
    module
        .runtime_scene_system(SYSTEM_ID, SystemStage::PostUpdate, || {
            |_context| Ok::<_, CoreError>(())
        })
        .in_set(SYSTEM_SET)
        .after(SystemRef::System(WORLD_TRANSFORM_SYSTEM.to_string()))
        .with_order(7)
        .with_tick_policy(SceneSystemTickPolicy::monotonic_real())
        .register()
        .expect("runtime scene system registered");
    module
        .runtime_scene_system(DEFAULT_DOMAIN_SYSTEM_ID, SystemStage::Update, || {
            |_context| Ok::<_, CoreError>(())
        })
        .register()
        .expect("default-domain runtime scene system registered");
    module
        .plugin_option(PluginOptionManifest::new(
            OPTION_ID,
            "SDK Registration Option",
            "bool",
            "false",
        ))
        .expect("plugin option registered");
    module
        .plugin_event_catalog(PluginEventCatalogManifest {
            namespace: CATALOG_NAMESPACE.to_string(),
            version: 1,
            events: vec![PluginEventManifest {
                id: CATALOG_SEED_EVENT_ID.to_string(),
                display_name: "SDK Registration Seed Event".to_string(),
                payload_schema: CATALOG_SEED_EVENT_SCHEMA.to_string(),
            }],
        })
        .expect("plugin event catalog registered");
    module
        .event::<SdkRegistrationEvent>(PluginEventManifest {
            id: EVENT_ID.to_string(),
            display_name: "SDK Registration Event".to_string(),
            payload_schema: EVENT_SCHEMA.to_string(),
        })
        .expect("runtime event registered");
    drop(module);
    registry.finalize();
    assert_eq!(
        imported.call(|_| ()),
        Err(zircon_runtime::core::framework::bridge::BridgeError::Absent)
    );
    assert!(registry
        .components()
        .iter()
        .any(|component| component.type_id == "sdk_registration.weather"));

    assert!(registry
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));

    let systems = registry.plugin_runtime_systems().collect::<Vec<_>>();
    assert_eq!(systems.len(), 2);
    let (owner, system) = systems
        .iter()
        .copied()
        .find(|(_, system)| system.id == SYSTEM_ID)
        .expect("explicit real-domain runtime system registered");
    assert_eq!(owner, module_owner);
    assert_eq!(registry.plugin_module_name(owner), Some(MODULE_OWNER));
    assert_eq!(system.id, SYSTEM_ID);
    assert_eq!(system.stage, SystemStage::PostUpdate);
    assert_eq!(system.order, 7);
    assert_eq!(
        system.tick_policy.clock_domain(),
        SceneSystemClockDomain::MonotonicReal
    );
    assert_eq!(system.sets.len(), 1);
    assert_eq!(
        system.constraints,
        vec![SystemOrderingConstraint::After(SystemRef::System(
            WORLD_TRANSFORM_SYSTEM.to_string()
        ))]
    );
    let (_, default_domain_system) = systems
        .iter()
        .copied()
        .find(|(_, system)| system.id == DEFAULT_DOMAIN_SYSTEM_ID)
        .expect("default-domain runtime system registered");
    assert_eq!(
        default_domain_system.tick_policy,
        SceneSystemTickPolicy::virtual_time()
    );

    let events = registry.plugin_events().collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    let (event_owner, event) = events[0];
    assert_eq!(registry.plugin_module_name(event_owner), Some(MODULE_OWNER));
    assert_eq!(event.manifest().id, EVENT_ID);

    let resources = registry.plugin_resources().collect::<Vec<_>>();
    assert_eq!(resources.len(), 1);
    assert_eq!(
        registry.plugin_module_name(resources[0].0),
        Some(MODULE_OWNER)
    );
    assert_eq!(
        resources[0].1.type_name(),
        std::any::type_name::<SdkRegistrationResource>()
    );

    assert!(registry
        .plugin_options()
        .iter()
        .any(|option| option.key == OPTION_ID));
    let event_catalog = registry
        .plugin_event_catalogs()
        .iter()
        .find(|catalog| catalog.namespace == CATALOG_NAMESPACE)
        .expect("event catalog registered");
    assert!(event_catalog
        .events
        .iter()
        .any(|event| event.id == EVENT_ID));
}

#[test]
fn runtime_registration_rejects_non_fixed_tick_policy_for_fixed_stages() {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
        .module(MODULE_OWNER)
        .expect("module registered");

    let error = module
        .runtime_scene_system(
            "sdk_registration.runtime.invalid-fixed-real",
            SystemStage::FixedUpdate,
            || |_context| Ok::<_, CoreError>(()),
        )
        .with_tick_policy(SceneSystemTickPolicy::monotonic_real())
        .register()
        .expect_err("fixed stages must reject the real-time clock domain");

    assert!(matches!(
        error,
        RuntimeExtensionRegistryError::InvalidPluginSystem(message)
            if message.contains("invalid-fixed-real")
                && message.contains("FixedUpdate")
                && message.contains("Real")
    ));
}

#[test]
fn component_registration_rejects_foreign_owner_and_revokes_with_its_module() {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
        .module("owner_a.runtime")
        .expect("module registered");

    let error = module
        .component(ComponentTypeDescriptor::new(
            "owner_b.Component.Foreign",
            "owner_b",
            "Foreign",
        ))
        .expect_err("builder must reject caller-forged component ownership");
    assert!(error.to_string().contains("owner_a"));
    assert!(error.to_string().contains("owner_b"));

    module
        .component(ComponentTypeDescriptor::new(
            "owner_a.Component.Local",
            "owner_a",
            "Local",
        ))
        .expect("matching owner component registered");
    let owner = module.owner();
    drop(module);

    assert_eq!(registry.components().len(), 1);
    let removed = registry.revoke_owner_registrations(owner);
    assert_eq!(removed.components.len(), 1);
    assert!(registry.components().is_empty());
}

#[test]
fn sdk_resource_registration_supports_concurrent_world_initialization() {
    let active_calls = Arc::new(AtomicUsize::new(0));
    let peak_active_calls = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
        .module("sdk_registration.concurrent")
        .expect("module registered");
    let active_calls_for_factory = Arc::clone(&active_calls);
    let peak_active_calls_for_factory = Arc::clone(&peak_active_calls);

    module
        .resource(move || {
            let active = active_calls_for_factory.fetch_add(1, Ordering::SeqCst) + 1;
            peak_active_calls_for_factory.fetch_max(active, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_millis(250);
            while active_calls_for_factory.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
                thread::yield_now();
            }
            thread::sleep(Duration::from_millis(20));
            active_calls_for_factory.fetch_sub(1, Ordering::SeqCst);
            SdkRegistrationResource
        })
        .expect("concurrent resource factory registered");
    drop(module);

    let registry = Arc::new(registry);
    let start = Arc::new(Barrier::new(2));
    thread::scope(|scope| {
        for _ in 0..2 {
            let registry = Arc::clone(&registry);
            let start = Arc::clone(&start);
            scope.spawn(move || {
                start.wait();
                let mut local_registry = registry.as_ref().clone();
                let mut world = zircon_runtime::scene::World::new();
                local_registry
                    .apply_to_world(&mut world)
                    .expect("SDK resource registration should apply to each world");
                assert!(world.contains_resource::<SdkRegistrationResource>());
            });
        }
    });

    assert_eq!(peak_active_calls.load(Ordering::SeqCst), 2);
    assert_eq!(active_calls.load(Ordering::SeqCst), 0);
}
