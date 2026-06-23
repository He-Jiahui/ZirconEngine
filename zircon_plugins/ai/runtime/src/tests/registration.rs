use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};

use crate::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin_descriptor,
    AI_DIST_CRATE_NAME, AI_DIST_RUNTIME_ENTRY, AI_MODULE_NAME, RUNTIME_CAPABILITIES,
};

#[test]
fn ai_registration_contributes_runtime_module_and_capabilities() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == AI_MODULE_NAME));
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert_eq!(
        report.package_manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        report.package_manifest.capabilities,
        runtime_capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    for capability in runtime_capabilities() {
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == *capability
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
    }
}

#[test]
fn ai_runtime_descriptor_matches_builtin_catalog_row() {
    let descriptor = runtime_plugin_descriptor();
    let catalog_descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Ai)
        .expect("AI built-in catalog entry");

    assert_eq!(catalog_descriptor.package_id(), descriptor.package_id());
    assert_eq!(catalog_descriptor.crate_name(), descriptor.crate_name());
    assert_eq!(catalog_descriptor.category(), descriptor.category());
    assert_eq!(catalog_descriptor.maturity(), descriptor.maturity());
    assert_eq!(catalog_descriptor.target_modes(), descriptor.target_modes());
    assert_eq!(catalog_descriptor.capabilities(), descriptor.capabilities());
    assert_eq!(
        catalog_descriptor.capability_statuses(),
        descriptor.capability_statuses()
    );
}

#[test]
fn ai_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("ai dist distribution");

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, AI_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, AI_DIST_RUNTIME_ENTRY);
    assert!(manifest.modules.iter().any(|module| {
        module.kind == zircon_runtime::plugin::PluginModuleKind::Native
            && module.name == "ai.dist"
            && module.crate_name == AI_DIST_CRATE_NAME
            && module.capabilities == RUNTIME_CAPABILITIES
    }));
}
