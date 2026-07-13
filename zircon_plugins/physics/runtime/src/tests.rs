use zircon_runtime::core::CoreRuntime;

use super::*;

#[test]
fn physics_step_duration_is_published_to_diagnostic_store() {
    use std::time::Duration;

    let runtime = zircon_runtime::core::CoreRuntime::new();
    let core = runtime.handle();
    record_physics_step_diagnostic(&core, 41, Duration::from_micros(2_500));

    let snapshot = core.diagnostic_store_snapshot();
    let series = snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == PHYSICS_STEP_DURATION_DIAGNOSTIC_PATH)
        .expect("physics step diagnostic path should be registered by publishing a sample");
    assert_eq!(series.current, Some(2.5));
    assert_eq!(series.unit.as_deref(), Some("ms"));
    assert_eq!(series.subsystem_tags, ["physics", "step"]);
}

#[test]
fn physics_registration_contributes_runtime_module() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == PHYSICS_MODULE_NAME));
    assert!(report
        .extensions
        .plugin_runtime_systems()
        .any(|(owner, system)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && system.id == PHYSICS_STEP_SYSTEM
                && system.stage == zircon_runtime::scene::SystemStage::FixedUpdate
        }));
    assert!(report
        .extensions
        .plugin_runtime_systems()
        .any(|(owner, system)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && system.id == "physics.sync_to_scene"
                && system.stage == zircon_runtime::scene::SystemStage::FixedPostUpdate
        }));
    for resource_type in [
        std::any::type_name::<zircon_runtime::core::framework::physics::SkeletalPoseTargets>(),
        std::any::type_name::<zircon_runtime::core::framework::physics::SimulatedPoseFeed>(),
        std::any::type_name::<RagdollRuntime>(),
    ] {
        assert!(report
            .extensions
            .plugin_resources()
            .any(|(owner, resource)| {
                report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                    && resource.type_name() == resource_type
            }));
    }
    for (event_type, event_id, payload_schema) in [
        (
            std::any::type_name::<zircon_runtime::core::framework::physics::PhysicsContactEvent>(),
            PHYSICS_CONTACT_EVENT_ID,
            PHYSICS_CONTACT_EVENT_SCHEMA,
        ),
        (
            std::any::type_name::<zircon_runtime::core::framework::physics::PhysicsTriggerEvent>(),
            PHYSICS_TRIGGER_EVENT_ID,
            PHYSICS_TRIGGER_EVENT_SCHEMA,
        ),
    ] {
        assert!(report.extensions.plugin_events().any(|(owner, event)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && event.type_name() == event_type
                && event.manifest().id == event_id
                && event.manifest().payload_schema == payload_schema
        }));
    }
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        vec!["physics.main".to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec![
            PHYSICS_STEP_SYSTEM.to_string(),
            "physics.sync_to_scene".to_string(),
        ]
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    for capability in [
        "runtime.plugin.physics",
        "runtime.capability.physics.raycast",
        "runtime.capability.physics.overlap",
        "runtime.capability.physics.shape_cast",
        "runtime.capability.physics.trigger_events",
        "runtime.capability.physics.constraints",
        "runtime.capability.physics.skeletal_joints",
    ] {
        assert!(report
            .package_manifest
            .capabilities
            .contains(&capability.to_string()));
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == capability
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
    }
}

#[test]
fn physics_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("physics distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, PHYSICS_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, PHYSICS_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "physics.dist")
        .expect("physics native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, PHYSICS_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}

#[test]
fn physics_module_resolves_manager() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(PHYSICS_MODULE_NAME).unwrap();

    runtime
        .handle()
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap();
}
