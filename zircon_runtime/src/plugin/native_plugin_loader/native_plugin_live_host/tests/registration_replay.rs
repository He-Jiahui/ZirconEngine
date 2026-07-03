use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::super::behavior_calls::NativePluginBehavior;
use super::*;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::{SystemStage, World};

static DIST_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn dist_system_plugin_loads_and_ticks_via_bridge() {
    DIST_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    let load_report = NativePluginLoadReport {
        discovered: Vec::new(),
        loaded: vec![native_live_host_dist_system_plugin()],
        diagnostics: Vec::new(),
    };
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
    host.load_reported_plugins(
        NativePluginLoadReport {
            discovered: Vec::new(),
            loaded: vec![native_live_host_dist_system_plugin()],
            diagnostics: Vec::new(),
        },
        PluginModuleKind::Runtime,
    )
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
            invoke_command: None,
            save_state: None,
            restore_state: None,
            unload: None,
        });
    }
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
