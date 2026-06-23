use zircon_runtime::core::CoreRuntime;

use super::*;

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
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        vec![PHYSICS_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec![PHYSICS_STEP_SYSTEM.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
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

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("physics distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
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
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
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
