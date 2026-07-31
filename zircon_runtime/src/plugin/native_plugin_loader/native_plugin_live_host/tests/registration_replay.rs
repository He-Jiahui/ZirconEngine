use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::super::super::behavior_calls::NativePluginBehavior;
use super::super::super::registration_manifest::{
    NativeSystemAccessAuthorityError, NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY,
};
use super::*;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::PluginModuleManifest;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::{SystemStage, World};

static DIST_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static OLD_GENERATION_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static RELOADED_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn native_live_host_uses_typed_borrowed_plugin_keys() {
    let source = include_str!("../keys.rs");

    assert!(source.contains("NativePluginLiveKey"));
    assert!(source.contains("NativePluginLiveRegistry"));
    assert!(!source.contains("format!(\"{}"));
}

#[test]
fn native_registration_replay_generation_borrows_build_inputs_without_full_clones() {
    let source = include_str!("../bridge_methods.rs");
    let start = source
        .find("pub(super) fn runtime_registration_replay_bridge_context_result")
        .expect("replay generation context builder must remain defined");
    let end = source[start..]
        .find("pub fn reload_runtime_bridge_provider_and_scope_from_installed_bindings")
        .map(|offset| start + offset)
        .expect("replay generation context builder must end before the reload helper");
    let context_builder = &source[start..end];

    assert!(context_builder.contains("avoiding a full package-manifest or binding-vector clone"));
    assert!(context_builder.contains("bindings.iter().cloned()"));
    assert!(!context_builder.contains("loaded_runtime_package_manifest_and_callback_owner_result"));
}

#[test]
fn native_live_host_typed_registry_keeps_module_kinds_distinct() {
    let mut registry = super::super::keys::NativePluginLiveRegistry::default();
    registry.insert(live_key(PluginModuleKind::Runtime, "physics"), 1_u8);
    registry.insert(live_key(PluginModuleKind::Editor, "physics"), 2_u8);

    assert_eq!(
        registry.get(&live_key(PluginModuleKind::Runtime, "physics")),
        Some(&1)
    );
    assert_eq!(
        registry.get(&live_key(PluginModuleKind::Editor, "physics")),
        Some(&2)
    );
}

#[test]
fn dist_system_plugin_loads_and_ticks_via_bridge() {
    DIST_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_live_host_dist_system_plugin());
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("runtime load report should install discovered bridge bindings");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut registry = RuntimeExtensionRegistry::default();

    let replay = host
        .replay_runtime_registration_manifests_via_bridge(&mut registry, &lifecycle)
        .expect("registration manifest should replay into runtime registry");

    assert!(replay.is_clean());
    assert_eq!(replay.registered_systems.len(), 1);
    assert_eq!(replay.registered_systems[0].plugin_id, "physics");
    assert_eq!(replay.registered_systems[0].module, "runtime");
    assert_eq!(
        replay.registered_systems[0].system_id,
        "physics.runtime_tick"
    );
    assert_eq!(replay.registered_systems[0].stage, SystemStage::Update);
    let systems = registry.plugin_systems().collect::<Vec<_>>();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].1.id, "physics.runtime_tick");
    assert_eq!(systems[0].1.stage, SystemStage::Update);

    let mut world = World::default();
    registry
        .apply_to_world(&mut world)
        .expect("replayed registration manifest should apply to a world");
    assert_eq!(
        world
            .scheduled_native_system_steps_for_stage(SystemStage::Update)
            .len(),
        1
    );
    assert!(world.run_native_scene_system("physics.runtime_tick"));
    assert_eq!(DIST_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn native_registration_replay_reports_typed_schema_error() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_registration_replay_plugin_with_schema(
                "physics",
                "zircon.native.registration-manifest/2",
            ),
        );
    }

    let error = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut RuntimeExtensionRegistry::default(),
            &native_live_host_bridge_lifecycle_state(false),
            "physics",
        )
        .expect_err("unsupported registration manifest schema should be typed");

    assert!(matches!(
        error,
        NativePluginRegistrationReplayError::UnsupportedManifestSchema {
            plugin_id,
            actual,
            expected: "zircon.native.registration-manifest/3"
        } if plugin_id == "physics" && actual == "zircon.native.registration-manifest/2"
    ));
}

#[test]
fn native_registration_replay_reports_typed_duplicate_system_error() {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_live_host_dist_system_plugin());
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("test should load the native runtime plugin");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut registry = RuntimeExtensionRegistry::default();
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut registry,
        &lifecycle,
        "physics",
    )
    .expect("first replay should register the native system");

    let error = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut registry,
            &lifecycle,
            "physics",
        )
        .expect_err("second replay should return typed duplicate system error");

    assert!(matches!(
        error,
        NativePluginRegistrationReplayError::RegisterNativeSystem {
            plugin_id,
            system_id,
            source: RuntimeExtensionRegistryError::DuplicatePluginSystem(source_system_id)
        } if plugin_id == "physics"
            && system_id == "physics.runtime_tick"
            && source_system_id == "physics.runtime_tick"
    ));
}

#[test]
fn native_registration_replay_compiles_authorized_worker_access() {
    let host = NativePluginLiveHost::default();
    let load_report = NativePluginLoadReport::from_loaded(vec![native_worker_access_plugin(true)]);
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .unwrap();
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_component(ComponentTypeDescriptor::new(
            "physics.Body",
            "physics",
            "Physics Body",
        ))
        .unwrap();

    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut registry,
        &native_live_host_bridge_lifecycle_state(false),
        "physics",
    )
    .unwrap();
    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();
    let system = world
        .schedule()
        .native_systems_for_stage(SystemStage::Update)
        .next()
        .unwrap();

    assert_eq!(
        system.thread_affinity(),
        crate::scene::ecs::SceneSystemThreadAffinity::WorkerSafe
    );
    assert!(system.supports_worldless_execution());
    assert!(!system.access().has_conservative_world_access());
    assert!(world
        .registered_external_resource_id("physics.solver")
        .is_some());
}

#[test]
fn native_registration_replay_rejects_worker_access_without_host_grant() {
    let host = NativePluginLiveHost::default();
    let load_report = NativePluginLoadReport::from_loaded(vec![native_worker_access_plugin(false)]);
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .unwrap();

    let error = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut RuntimeExtensionRegistry::default(),
            &native_live_host_bridge_lifecycle_state(false),
            "physics",
        )
        .expect_err("worker-safe manifest must not trust a capability the host did not grant");

    assert!(matches!(
        error,
        NativePluginRegistrationReplayError::InvalidSystemAccessAuthority {
            plugin_id,
            system_id,
            source: NativeSystemAccessAuthorityError::WorkerSafeCapabilityNotGranted,
        } if plugin_id == "physics" && system_id == "physics.runtime_tick"
    ));
}

#[test]
fn native_registration_replay_builds_one_frozen_bridge_context_per_plugin() {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_registration_replay_scale_plugin(8, 4));
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("scale fixture should load");

    let replay = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut RuntimeExtensionRegistry::default(),
            &native_live_host_bridge_lifecycle_state(false),
            "physics",
        )
        .expect("scale fixture should replay");
    let counts = host.registration_replay_context_build_counts();

    assert_eq!(replay.registered_systems.len(), 8);
    assert_eq!(counts.registration_manifest_parses, 1);
    assert_eq!(counts.registration_system_preparations, 8);
    assert_eq!(counts.package_manifest_snapshots, 1);
    assert_eq!(counts.binding_snapshots, 1);
    assert_eq!(counts.method_lookup_builds, 1);
    assert_eq!(counts.bridge_call_scope_builds, 1);
}

#[test]
fn native_registration_replay_reuses_a_generation_until_its_bindings_change() {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_registration_replay_generation_plugin());
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("initial binding generation should load");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);

    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &lifecycle,
        "physics",
    )
    .expect("initial generation should replay");
    let first_generation = host.registration_replay_context_build_counts();
    assert_eq!(first_generation.registration_manifest_parses, 1);
    assert_eq!(first_generation.registration_system_preparations, 1);
    assert_eq!(first_generation.package_manifest_snapshots, 1);
    assert_eq!(first_generation.binding_snapshots, 1);
    assert_eq!(first_generation.method_lookup_builds, 1);
    assert_eq!(first_generation.bridge_call_scope_builds, 1);

    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &lifecycle,
        "physics",
    )
    .expect("unchanged generation should replay from its cache");
    assert_eq!(
        host.registration_replay_context_build_counts(),
        first_generation,
        "an unchanged plugin must reuse its parsed manifest, validated bindings, slots, and call scope"
    );

    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_registration_replay_reloaded_bridge_method),
        )],
    )
    .expect("replacement binding generation should install");
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &lifecycle,
        "physics",
    )
    .expect("replacement generation should replay");
    let replacement_generation = host.registration_replay_context_build_counts();
    assert_eq!(replacement_generation.registration_manifest_parses, 2);
    assert_eq!(replacement_generation.registration_system_preparations, 2);
    assert_eq!(replacement_generation.package_manifest_snapshots, 2);
    assert_eq!(replacement_generation.binding_snapshots, 2);
    assert_eq!(replacement_generation.method_lookup_builds, 2);
    assert_eq!(replacement_generation.bridge_call_scope_builds, 2);
}

#[test]
fn native_registration_replay_rebuilds_generation_for_a_different_lifecycle_bridge_table() {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_registration_replay_generation_plugin());
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("initial binding generation should load");

    let first_lifecycle = native_live_host_bridge_lifecycle_state(false);
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &first_lifecycle,
        "physics",
    )
    .expect("first lifecycle generation should replay");
    let first_generation = host.registration_replay_context_build_counts();
    assert_eq!(first_generation.registration_manifest_parses, 1);
    assert_eq!(first_generation.bridge_call_scope_builds, 1);

    let second_lifecycle = native_live_host_bridge_lifecycle_state(false);
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &second_lifecycle,
        "physics",
    )
    .expect("second lifecycle generation should replay");
    let second_generation = host.registration_replay_context_build_counts();
    assert_eq!(second_generation.registration_manifest_parses, 2);
    assert_eq!(second_generation.registration_system_preparations, 2);
    assert_eq!(second_generation.package_manifest_snapshots, 2);
    assert_eq!(second_generation.binding_snapshots, 2);
    assert_eq!(second_generation.method_lookup_builds, 2);
    assert_eq!(second_generation.bridge_call_scope_builds, 2);
}

#[test]
fn native_registration_replay_keeps_old_binding_generation_alive_after_reinstall() {
    OLD_GENERATION_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    RELOADED_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_registration_replay_generation_plugin());
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("initial binding generation should load");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut old_registry = RuntimeExtensionRegistry::default();
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut old_registry,
        &lifecycle,
        "physics",
    )
    .expect("old binding generation should replay");

    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_registration_replay_reloaded_bridge_method),
        )],
    )
    .expect("replacement binding generation should install");
    let mut new_registry = RuntimeExtensionRegistry::default();
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut new_registry,
        &lifecycle,
        "physics",
    )
    .expect("replacement binding generation should replay");

    let mut old_world = World::default();
    old_registry
        .apply_to_world(&mut old_world)
        .expect("old binding generation should apply");
    let mut new_world = World::default();
    new_registry
        .apply_to_world(&mut new_world)
        .expect("replacement binding generation should apply");
    assert!(old_world.run_native_scene_system("physics.runtime_tick"));
    assert!(new_world.run_native_scene_system("physics.runtime_tick"));

    assert_eq!(
        OLD_GENERATION_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
        1
    );
    assert_eq!(RELOADED_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
#[ignore = "manual native registration replay scale evidence"]
fn native_registration_replay_scale_benchmark_builds_one_context_per_plugin() {
    for system_count in [1, 100, 1_000] {
        for method_count in [1, 100] {
            let host = NativePluginLiveHost::default();
            let mut load_report = NativePluginLoadReport::default();
            load_report.push_loaded(native_registration_replay_scale_plugin(
                system_count,
                method_count,
            ));
            host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
                .expect("scale fixture should load");
            let started = Instant::now();

            let replay = host
                .replay_runtime_plugin_registration_manifest_via_bridge_result(
                    &mut RuntimeExtensionRegistry::default(),
                    &native_live_host_bridge_lifecycle_state(false),
                    "physics",
                )
                .expect("scale fixture should replay");
            let elapsed = started.elapsed();
            let counts = host.registration_replay_context_build_counts();

            assert_eq!(replay.registered_systems.len(), system_count);
            assert_eq!(counts.registration_manifest_parses, 1);
            assert_eq!(counts.registration_system_preparations, system_count);
            assert_eq!(counts.package_manifest_snapshots, 1);
            assert_eq!(counts.binding_snapshots, 1);
            assert_eq!(counts.method_lookup_builds, 1);
            assert_eq!(counts.bridge_call_scope_builds, 1);
            eprintln!(
                "native registration replay: systems={system_count} methods={method_count} \
                 elapsed_us={} registration_manifest_parses=1 prepared_systems={} manifest_snapshots=1 binding_snapshots=1 method_lookup_builds=1 \
                 bridge_call_scope_builds=1",
                elapsed.as_micros(),
                system_count,
            );
        }
    }
}

fn native_live_host_dist_system_plugin() -> LoadedNativePlugin {
    let mut plugin = native_live_host_test_plugin_with_bridge_manifest("physics");
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report
            .bridge_method_bindings
            .push(NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_rust(native_live_host_dist_system_bridge_method),
            ));
        report.behavior = Some(NativePluginBehavior {
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: Some("zircon.native.registration-manifest/3".to_string()),
            command_manifest: None,
            event_manifest: None,
            registration_manifest: Some(
                r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics"]

[[modules]]
name = "runtime"
kind = "runtime"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
order = 1
sets = ["physics.tick"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#
                .to_string(),
            ),
            command_table: None,
            invoke_command: None,
            save_state: None,
            restore_state: None,
            unload: None,
        });
    }
    plugin
}

fn native_worker_access_plugin(grant_worker_safe: bool) -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    let interface =
        PluginInterfaceManifest::new(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID)
            .with_method(PluginInterfaceMethodManifest::new("sample_count", 7));
    let mut runtime_module = PluginModuleManifest::runtime("physics.runtime", "physics_runtime");
    if grant_worker_safe {
        runtime_module =
            runtime_module.with_capabilities([NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()]);
    }
    let manifest = PluginPackageManifest::new("physics", "Physics")
        .with_runtime_module(runtime_module)
        .with_component(ComponentTypeDescriptor::new(
            "physics.Body",
            "physics",
            "Physics Body",
        ))
        .with_provided_interface(interface);
    if let Some(descriptor) = plugin.descriptor.as_mut() {
        descriptor.package_manifest = Some(manifest.clone());
        if grant_worker_safe {
            descriptor
                .requested_capabilities
                .push(NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string());
        }
    }
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report.package_manifest = Some(manifest);
        report
            .behavior
            .as_mut()
            .expect("worker access fixture behavior")
            .registration_manifest = Some(
            r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics", "runtime.native.system.worker_safe"]

[[modules]]
name = "runtime"
kind = "runtime"

[[resources]]
id = "physics.solver"
module = "runtime"
schema = "physics.solver/v1"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
thread_affinity = "worker-safe"
access = ["read:component:physics.Body", "write:resource:physics.solver"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#
            .to_string(),
        );
    }
    plugin
}

fn native_registration_replay_scale_plugin(
    system_count: usize,
    method_count: usize,
) -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    let mut interface =
        PluginInterfaceManifest::new(<dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID);
    let mut bindings = Vec::with_capacity(method_count);
    for method_index in 0..method_count {
        let method_name = format!("method_{method_index}");
        interface = interface.with_method(PluginInterfaceMethodManifest::new(
            method_name.clone(),
            method_index as u32 + 1,
        ));
        bindings.push(NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            method_name,
            NativeBridgeMethodFn::from_rust(native_live_host_dist_system_bridge_method),
        ));
    }
    let manifest = PluginPackageManifest::new("physics", "Physics")
        .with_runtime_crate("physics_runtime")
        .with_provided_interface(interface);
    let mut registration_manifest = String::from(
        "schema = \"zircon.native.registration-manifest/3\"\n\
         capabilities = [\"runtime.plugin.physics\"]\n\n\
         [[modules]]\n\
         name = \"runtime\"\n\
         kind = \"runtime\"\n",
    );
    for system_index in 0..system_count {
        let method_index = system_index % method_count;
        registration_manifest.push_str(&format!(
            "\n[[systems]]\n\
             id = \"physics.runtime_tick_{system_index}\"\n\
             module = \"runtime\"\n\
             stage = \"Update\"\n\
             order = {system_index}\n\
             sets = [\"physics.tick\"]\n\
             access = [\"write:world\"]\n\
             bridge_interface = \"native.live_host.bridge.v1\"\n\
             bridge_method = \"method_{method_index}\"\n"
        ));
    }
    if let Some(descriptor) = plugin.descriptor.as_mut() {
        descriptor.package_manifest = Some(manifest.clone());
    }
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report.package_manifest = Some(manifest);
        report.bridge_method_bindings = bindings;
        report
            .behavior
            .as_mut()
            .expect("registration replay fixture behavior")
            .registration_manifest = Some(registration_manifest);
    }
    plugin
}

fn native_registration_replay_generation_plugin() -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    plugin
        .runtime_entry_report
        .as_mut()
        .expect("registration replay generation fixture report")
        .bridge_method_bindings = vec![NativeBridgeMethodBinding::new(
        <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
        "sample_count",
        NativeBridgeMethodFn::from_rust(native_registration_replay_old_generation_bridge_method),
    )];
    plugin
}

fn native_live_host_registration_replay_plugin_with_schema(
    plugin_id: &str,
    schema: &str,
) -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    plugin.plugin_id = plugin_id.to_string();
    if let Some(descriptor) = plugin.descriptor.as_mut() {
        descriptor.plugin_id = plugin_id.to_string();
    }
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report.plugin_id = plugin_id.to_string();
        if let Some(behavior) = report.behavior.as_mut() {
            behavior.registration_manifest_schema = Some(schema.to_string());
        }
    }
    plugin
}

fn native_live_host_dist_system_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        DIST_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}

fn native_registration_replay_reloaded_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        RELOADED_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}

fn native_registration_replay_old_generation_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        OLD_GENERATION_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}
