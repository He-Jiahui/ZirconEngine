use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::super::behavior_calls::NativePluginBehavior;
use super::*;
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

fn native_live_host_dist_system_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        DIST_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}
