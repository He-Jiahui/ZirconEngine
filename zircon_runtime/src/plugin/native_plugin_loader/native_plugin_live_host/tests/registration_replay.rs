use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use super::super::super::behavior_calls::NativePluginBehavior;
use super::super::super::benchmark_harness::{BenchmarkMeasurement, BenchmarkRunMetadata};
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
static INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);
static INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn native_live_host_uses_typed_borrowed_plugin_keys() {
    let source = include_str!("../keys.rs");

    assert!(source.contains("NativePluginLiveKey"));
    assert!(source.contains("NativePluginLiveRegistry"));
    assert!(!source.contains("format!(\"{}"));
}

#[test]
fn native_registration_replay_generation_uses_one_validated_bridge_binding_authority() {
    let source = include_str!("../bridge_methods.rs");
    let start = source
        .find("pub(super) fn build_runtime_bridge_generation_result")
        .expect("replay generation context builder must remain defined");
    let end = source[start..]
        .find("pub fn reload_runtime_bridge_provider_and_scope_from_installed_bindings")
        .map(|offset| start + offset)
        .expect("replay generation context builder must end before the reload helper");
    let context_builder = &source[start..end];

    assert!(source.contains("ValidatedRuntimeBridgeMethodBindings"));
    assert!(context_builder.contains("validated_bindings.descriptors.iter()"));
    assert!(context_builder.contains("validated_bindings.method_slots.clone()"));
    assert!(!context_builder.contains("bindings.iter().cloned()"));
    assert!(!context_builder.contains("loaded_runtime_package_manifest_and_callback_owner_result"));
    assert!(!source.contains("runtime_bridge_call_scope_from_loaded_manifest"));
}

#[test]
fn native_registration_replay_reads_manifest_and_callbacks_under_one_loaded_guard() {
    let source = include_str!("../registration_replay.rs");
    let start = source
        .find("fn build_runtime_registration_replay_generation")
        .expect("registration replay generation builder must remain defined");
    let end = source[start..]
        .find("fn replay_runtime_registration_system")
        .map(|offset| start + offset)
        .expect("registration replay generation builder must end before system replay");
    let builder = &source[start..end];

    let loaded_lock = builder
        .find("lock_loaded_native_plugins(&self.loaded)")
        .expect("generation build must acquire the loaded-generation guard");
    let manifest_source = builder
        .find("runtime_registration_manifest_source(plugin_id, plugin)")
        .expect("manifest source must be read from the guarded loaded plugin");
    let bridge_generation = builder
        .find("runtime_bridge_generation_result(plugin_id, lifecycle)")
        .expect("bridge generation must be read while the loaded generation is guarded");
    let loaded_drop = builder
        .find("drop(loaded)")
        .expect("generation build must explicitly retain the loaded guard through preparation");

    assert!(loaded_lock < manifest_source);
    assert!(manifest_source < bridge_generation);
    assert!(bridge_generation < loaded_drop);
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
fn native_registration_replay_preserves_sets_order_and_before_after_constraints() {
    let host = NativePluginLiveHost::default();
    let load_report =
        NativePluginLoadReport::from_loaded(vec![native_live_host_ordered_systems_plugin()]);
    host.load_reported_plugins(load_report, PluginModuleKind::Runtime)
        .expect("ordered runtime plugin should load");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut registry = RuntimeExtensionRegistry::default();

    let replay = host
        .replay_runtime_registration_manifests_via_bridge(&mut registry, &lifecycle)
        .expect("ordered registration manifest should replay");
    assert!(replay.is_clean());
    assert_eq!(
        replay
            .registered_systems
            .iter()
            .map(|system| (system.system_id.as_str(), system.order))
            .collect::<Vec<_>>(),
        vec![
            ("physics.bootstrap", 100),
            ("physics.runtime_tick", 0),
            ("physics.render", -100),
        ]
    );
    let expected_tick_set = registry
        .intern_system_set("physics.tick")
        .expect("replayed set should retain its exact interned identity");

    let mut world = World::default();
    registry
        .apply_to_world(&mut world)
        .expect("ordered replay should compile into the scene schedule");
    let tick = world
        .schedule()
        .native_systems_for_stage(SystemStage::Update)
        .find(|system| system.id() == "physics.runtime_tick")
        .expect("runtime tick system should be registered");
    assert_eq!(tick.order(), 0);
    assert_eq!(tick.sets(), &[expected_tick_set]);
    assert_eq!(tick.constraints().len(), 2);

    let scheduled_ids = world
        .scheduled_native_system_steps_for_stage(SystemStage::Update)
        .into_iter()
        .filter_map(|step| match step {
            crate::scene::ecs::ScheduledSceneStep::Native { id, .. } => Some(id),
            crate::scene::ecs::ScheduledSceneStep::Runtime { .. }
            | crate::scene::ecs::ScheduledSceneStep::ApplyDeferred { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        scheduled_ids,
        vec![
            "physics.bootstrap".to_string(),
            "physics.runtime_tick".to_string(),
            "physics.render".to_string(),
        ],
        "before/after constraints must dominate conflicting numeric order values"
    );
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
    assert_eq!(counts.package_manifest_snapshots, 0);
    assert_eq!(counts.binding_snapshots, 0);
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
    assert_eq!(first_generation.package_manifest_snapshots, 0);
    assert_eq!(first_generation.binding_snapshots, 0);
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
    assert_eq!(replacement_generation.package_manifest_snapshots, 0);
    assert_eq!(replacement_generation.binding_snapshots, 0);
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
    assert_eq!(second_generation.package_manifest_snapshots, 0);
    assert_eq!(second_generation.binding_snapshots, 0);
    assert_eq!(second_generation.method_lookup_builds, 1);
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
fn native_registration_replay_and_reload_publish_both_consistent_generation_orders() {
    for order in [
        ReplayReloadPublicationOrder::CacheBeforeInvalidate,
        ReplayReloadPublicationOrder::InvalidateBeforeCache,
    ] {
        assert_registration_replay_reload_publication_order(order);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayReloadPublicationOrder {
    CacheBeforeInvalidate,
    InvalidateBeforeCache,
}

fn assert_registration_replay_reload_publication_order(order: ReplayReloadPublicationOrder) {
    INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT.store(0, Ordering::SeqCst);
    let host = Arc::new(NativePluginLiveHost::default());
    host.load_reported_plugins(
        NativePluginLoadReport::from_loaded(vec![native_registration_replay_interleaved_plugin(
            false,
        )]),
        PluginModuleKind::Runtime,
    )
    .expect("initial interleaved generation should load");
    let lifecycle = Arc::new(native_live_host_bridge_lifecycle_state(false));
    let mut source_gate = host.install_registration_replay_source_test_gate();
    let mut before_cache_gate = (order == ReplayReloadPublicationOrder::InvalidateBeforeCache)
        .then(|| host.install_registration_replay_before_cache_test_gate());

    let replay_host = host.clone();
    let replay_lifecycle = lifecycle.clone();
    let replay = thread::spawn(move || {
        replay_and_run_interleaved_registration(&replay_host, &replay_lifecycle)
    });
    source_gate.wait_until_reached("registration replay source capture");

    let lock_attempts_before_reload = host.loaded.lock_attempts();
    let mut after_lock_gate = (order == ReplayReloadPublicationOrder::CacheBeforeInvalidate)
        .then(|| host.loaded.install_after_lock_test_gate());
    let reload_host = host.clone();
    let (reload_complete_tx, reload_complete_rx) = mpsc::channel();
    let reload = thread::spawn(move || {
        let result = reload_host
            .load_reported_plugins(
                NativePluginLoadReport::from_loaded(vec![
                    native_registration_replay_interleaved_plugin(true),
                ]),
                PluginModuleKind::Runtime,
            )
            .map(|_| ());
        reload_complete_tx
            .send(result)
            .expect("reload completion receiver should remain alive");
    });
    let wait_started = Instant::now();
    while host.loaded.lock_attempts() == lock_attempts_before_reload {
        assert!(
            wait_started.elapsed() < Duration::from_secs(5),
            "reload should attempt the loaded-generation lock while replay is paused"
        );
        thread::yield_now();
    }
    assert!(
        matches!(
            reload_complete_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ),
        "reload must not publish while replay retains the loaded-generation guard"
    );

    source_gate.resume();
    let first_system_id = match order {
        ReplayReloadPublicationOrder::CacheBeforeInvalidate => {
            let after_lock_gate = after_lock_gate
                .as_mut()
                .expect("cache-first order should install an after-lock gate");
            after_lock_gate.wait_until_reached("reload loaded-lock acquisition");
            let system_id = replay.join().expect("replay thread should not panic");
            assert_eq!(system_id, "physics.old_generation_tick");
            assert_eq!(
                (
                    INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
                    INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
                ),
                (1, 0),
                "cache-first replay must pair the old manifest with the still-admitted old callback"
            );
            after_lock_gate.resume();
            reload_complete_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("cache-first reload should complete after its loaded-lock gate resumes")
                .expect("cache-first replacement generation should load");
            system_id
        }
        ReplayReloadPublicationOrder::InvalidateBeforeCache => {
            let before_cache_gate = before_cache_gate
                .as_mut()
                .expect("invalidate-first order should install a before-cache gate");
            before_cache_gate.wait_until_reached("registration replay cache publication");
            reload_complete_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("invalidate-first reload should complete before replay publishes its cache")
                .expect("invalidate-first replacement generation should load");
            before_cache_gate.resume();
            let system_id = replay.join().expect("replay thread should not panic");
            assert_eq!(system_id, "physics.reloaded_generation_tick");
            assert_eq!(
                (
                    INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
                    INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
                ),
                (0, 1),
                "invalidate-first replay must reject the old cache and pair the new manifest with the new callback"
            );
            system_id
        }
    };
    reload.join().expect("reload thread should not panic");

    for _ in 0..2 {
        assert_eq!(
            replay_and_run_interleaved_registration(&host, &lifecycle),
            "physics.reloaded_generation_tick"
        );
    }
    assert_eq!(
        host.registration_replay_context_build_counts()
            .registration_manifest_parses,
        2,
        "old and replacement generations should each parse once; subsequent replay must hit the cache"
    );
    assert_eq!(
        INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
        usize::from(first_system_id == "physics.old_generation_tick")
    );
    assert_eq!(
        INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT.load(Ordering::SeqCst),
        usize::from(first_system_id == "physics.reloaded_generation_tick") + 2
    );
}

fn replay_and_run_interleaved_registration(
    host: &NativePluginLiveHost,
    lifecycle: &RuntimePluginBridgeLifecycleState,
) -> String {
    let mut registry = RuntimeExtensionRegistry::default();
    let report = host
        .replay_runtime_plugin_registration_manifest_via_bridge_result(
            &mut registry,
            lifecycle,
            "physics",
        )
        .expect("interleaved registration generation should replay");
    assert_eq!(report.registered_systems.len(), 1);
    let system_id = report.registered_systems[0].system_id.clone();
    let mut world = World::default();
    registry
        .apply_to_world(&mut world)
        .expect("interleaved replay generation should apply");
    assert!(world.run_native_scene_system(&system_id));
    system_id
}

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

fn native_live_host_ordered_systems_plugin() -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    let report = plugin
        .runtime_entry_report
        .as_mut()
        .expect("ordered test plugin should expose a runtime entry report");
    let behavior = report
        .behavior
        .as_mut()
        .expect("ordered test plugin should expose runtime behavior");
    behavior.registration_manifest = Some(
        r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics"]

[[modules]]
name = "runtime"
kind = "runtime"

[[systems]]
id = "physics.bootstrap"
module = "runtime"
stage = "Update"
order = 100
sets = ["physics.bootstrap"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"

[[systems]]
id = "physics.runtime_tick"
module = "runtime"
stage = "Update"
order = 0
sets = ["physics.tick"]
before = ["physics.render"]
after = ["physics.bootstrap"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"

[[systems]]
id = "physics.render"
module = "runtime"
stage = "Update"
order = -100
sets = ["physics.render"]
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#
        .to_string(),
    );
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

fn native_registration_replay_interleaved_plugin(reloaded: bool) -> LoadedNativePlugin {
    let mut plugin = native_live_host_dist_system_plugin();
    let report = plugin
        .runtime_entry_report
        .as_mut()
        .expect("interleaved registration replay fixture report");
    let (system_id, method) = if reloaded {
        (
            "physics.reloaded_generation_tick",
            native_registration_replay_interleaved_reloaded_bridge_method
                as fn(NativeBridgeCall) -> ZrStatus,
        )
    } else {
        (
            "physics.old_generation_tick",
            native_registration_replay_interleaved_old_bridge_method
                as fn(NativeBridgeCall) -> ZrStatus,
        )
    };
    report.bridge_method_bindings = vec![NativeBridgeMethodBinding::new(
        <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
        "sample_count",
        NativeBridgeMethodFn::from_rust(method),
    )];
    report
        .behavior
        .as_mut()
        .expect("interleaved registration replay fixture behavior")
        .registration_manifest = Some(format!(
        r#"
schema = "zircon.native.registration-manifest/3"
capabilities = ["runtime.plugin.physics"]

[[modules]]
name = "runtime"
kind = "runtime"

[[systems]]
id = "{system_id}"
module = "runtime"
stage = "Update"
access = ["write:world"]
bridge_interface = "native.live_host.bridge.v1"
bridge_method = "sample_count"
"#
    ));
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

fn native_registration_replay_interleaved_old_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        INTERLEAVED_OLD_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}

fn native_registration_replay_interleaved_reloaded_bridge_method(
    call: NativeBridgeCall,
) -> ZrStatus {
    let payload = unsafe { call.payload.as_slice() };
    if call.method_slot == 7 && payload.is_empty() {
        INTERLEAVED_RELOADED_SYSTEM_BRIDGE_TICK_COUNT.fetch_add(1, Ordering::SeqCst);
        ZrStatus::ok()
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}
