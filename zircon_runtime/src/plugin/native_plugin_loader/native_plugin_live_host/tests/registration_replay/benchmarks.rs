use std::time::Instant;

use super::super::super::super::benchmark_harness::{BenchmarkMeasurement, BenchmarkRunMetadata};
use super::*;

#[test]
#[ignore = "manual native registration replay benchmark: 1 system, 1 method"]
fn native_registration_replay_1_systems_1_methods_benchmark() {
    run_native_registration_replay_benchmark(1, 1);
}

#[test]
#[ignore = "manual native registration replay benchmark: 1 system, 100 methods"]
fn native_registration_replay_1_systems_100_methods_benchmark() {
    run_native_registration_replay_benchmark(1, 100);
}

#[test]
#[ignore = "manual native registration replay benchmark: 100 systems, 1 method"]
fn native_registration_replay_100_systems_1_methods_benchmark() {
    run_native_registration_replay_benchmark(100, 1);
}

#[test]
#[ignore = "manual native registration replay benchmark: 100 systems, 100 methods"]
fn native_registration_replay_100_systems_100_methods_benchmark() {
    run_native_registration_replay_benchmark(100, 100);
}

#[test]
#[ignore = "manual native registration replay benchmark: 1000 systems, 1 method"]
fn native_registration_replay_1000_systems_1_methods_benchmark() {
    run_native_registration_replay_benchmark(1_000, 1);
}

#[test]
#[ignore = "manual native registration replay benchmark: 1000 systems, 100 methods"]
fn native_registration_replay_1000_systems_100_methods_benchmark() {
    run_native_registration_replay_benchmark(1_000, 100);
}

fn run_native_registration_replay_benchmark(system_count: usize, method_count: usize) {
    let metadata = BenchmarkRunMetadata::from_environment(
        "native_registration_replay",
        format!("systems={system_count},methods={method_count}"),
    )
    .expect("benchmark metadata must be bound to a managed optimized-profile run");

    replay_native_registration_scale_fixture(system_count, method_count);
    let host = native_registration_replay_scale_host(system_count, method_count);
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut registry = RuntimeExtensionRegistry::default();
    let started = Instant::now();
    let replay = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut registry,
            &lifecycle,
            "physics",
        )
        .expect("scale fixture should replay");
    let elapsed = started.elapsed();
    let counts = host.registration_replay_context_build_counts();

    assert_eq!(replay.registered_systems.len(), system_count);
    assert_eq!(counts.registration_manifest_parses, 1);
    assert_eq!(counts.registration_system_preparations, system_count);
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
    assert_eq!(counts.method_lookup_builds, 1);
    assert_eq!(counts.bridge_call_scope_builds, 1);
    metadata.emit(BenchmarkMeasurement {
        warmup_operations: 1,
        measured_operations: 1,
        elapsed,
        counters: &[
            (
                "registration_manifest_parses",
                counts.registration_manifest_parses as u64,
            ),
            (
                "registration_system_preparations",
                counts.registration_system_preparations as u64,
            ),
            (
                "package_manifest_snapshots",
                counts.package_manifest_snapshots as u64,
            ),
            ("binding_snapshots", counts.binding_snapshots as u64),
            ("method_lookup_builds", counts.method_lookup_builds as u64),
            (
                "bridge_call_scope_builds",
                counts.bridge_call_scope_builds as u64,
            ),
        ],
        latency_sample: None,
    });
}

fn replay_native_registration_scale_fixture(system_count: usize, method_count: usize) {
    let host = native_registration_replay_scale_host(system_count, method_count);
    host.replay_runtime_plugin_registration_manifest_via_bridge_result(
        &mut RuntimeExtensionRegistry::default(),
        &native_live_host_bridge_lifecycle_state(false),
        "physics",
    )
    .expect("registration replay warm-up should succeed");
}

fn native_registration_replay_scale_host(
    system_count: usize,
    method_count: usize,
) -> NativePluginLiveHost {
    let host = NativePluginLiveHost::default();
    let mut load_report = NativePluginLoadReport::default();
    load_report.push_loaded(native_registration_replay_scale_plugin(
        system_count,
        method_count,
    ));
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("scale fixture should load");
    host
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
