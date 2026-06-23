use zircon_runtime::core::CoreRuntime;
use zircon_runtime::plugin::{ExportPackagingStrategy, PluginModuleKind};

use super::*;

#[test]
fn animation_registration_contributes_runtime_module() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == ANIMATION_MODULE_NAME));
    assert!(report
        .extensions
        .plugin_runtime_systems()
        .any(|(owner, system)| {
            report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                && system.id == ANIMATION_EVALUATE_SYSTEM
                && system.stage == zircon_runtime::scene::SystemStage::PostUpdate
        }));
    assert_eq!(
        report.package_manifest.modules[0].system_sets,
        vec![ANIMATION_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        report.package_manifest.modules[0].system_anchors,
        vec![ANIMATION_EVALUATE_SYSTEM.to_string()]
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
        zircon_runtime::plugin::PluginMaturity::Beta
    );
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == ANIMATION_RUNTIME_CAPABILITY
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                && status
                    .bevy_references
                    .iter()
                    .any(|reference| reference == "dev/bevy/crates/bevy_animation/src/lib.rs")
        }));
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
        }));
}

#[test]
fn animation_module_resolves_manager() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(ANIMATION_MODULE_NAME).unwrap();

    runtime
        .handle()
        .resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)
        .unwrap();
}

#[test]
fn animation_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("animation distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, ANIMATION_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, ANIMATION_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "animation.dist")
        .expect("animation native dist module");
    assert_eq!(native_module.kind, PluginModuleKind::Native);
    assert_eq!(native_module.crate_name, ANIMATION_DIST_CRATE_NAME);
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

    assert!(manifest
        .modules
        .iter()
        .any(|module| module.name == "animation.runtime"));
    assert_eq!(
        manifest.modules[0].system_sets,
        vec![ANIMATION_SYSTEM_SET.to_string()]
    );
    assert_eq!(
        manifest.modules[0].system_anchors,
        vec![ANIMATION_EVALUATE_SYSTEM.to_string()]
    );
}
