use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};

use crate::{plugin_registration, runtime_capabilities, runtime_plugin_descriptor, AI_MODULE_NAME};

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
        .find(|descriptor| descriptor.runtime_id == RuntimePluginId::Ai)
        .expect("AI built-in catalog entry");

    assert_eq!(catalog_descriptor.package_id, descriptor.package_id);
    assert_eq!(catalog_descriptor.crate_name, descriptor.crate_name);
    assert_eq!(catalog_descriptor.category, descriptor.category);
    assert_eq!(catalog_descriptor.maturity, descriptor.maturity);
    assert_eq!(catalog_descriptor.target_modes, descriptor.target_modes);
    assert_eq!(catalog_descriptor.capabilities, descriptor.capabilities);
    assert_eq!(
        catalog_descriptor.capability_statuses,
        descriptor.capability_statuses
    );
}
