use super::*;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::native_plugin_loader::registration_manifest::NativeSystemAccessAuthorityError;
use crate::plugin::RuntimeExtensionRegistryError;

#[test]
fn native_live_host_uses_typed_borrowed_plugin_keys() {
    let source = include_str!("../../keys.rs");

    assert!(source.contains("NativePluginLiveKey"));
    assert!(source.contains("NativePluginLiveRegistry"));
    assert!(!source.contains("format!(\"{}"));
}

#[test]
fn native_registration_replay_generation_uses_one_validated_bridge_binding_authority() {
    let source = include_str!("../../bridge_methods.rs");
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
    let source = include_str!("../../registration_replay.rs");
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
    let mut registry = super::super::super::keys::NativePluginLiveRegistry::default();
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
