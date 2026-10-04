use super::*;

#[test]
fn solari_registration_contributes_unavailable_provider_contract() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == SOLARI_MODULE_NAME));
    assert_eq!(
        report.extensions.solari_runtime_providers()[0].provider_id(),
        SOLARI_PROVIDER_ID
    );
    assert!(report
        .package_manifest
        .capabilities
        .contains(&SOLARI_CAPABILITY.to_string()));

    let provider = &report.extensions.solari_runtime_providers()[0];
    let availability = provider.provider().availability(provider.provider_id());
    assert_eq!(
        availability.provider_status,
        SolariRuntimeStatus::Unavailable
    );
    assert_eq!(
        availability.provider_message.as_deref(),
        Some(SOLARI_UNAVAILABLE_MESSAGE)
    );
}

#[test]
fn solari_runtime_capabilities_match_package_manifest() {
    let manifest = package_manifest();
    for capability in runtime_capabilities() {
        assert!(
            manifest.capabilities.contains(&capability.to_string()),
            "missing {capability}"
        );
        assert!(
            manifest.modules[0]
                .capabilities
                .contains(&capability.to_string()),
            "module missing {capability}"
        );
    }
}

#[test]
fn solari_package_manifest_declares_public_metadata() {
    let manifest = package_manifest();
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Runtime)
        .expect("solari runtime module");

    assert_eq!(manifest.category, "rendering");
    assert_eq!(
        manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert_eq!(
        manifest.supported_targets,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        manifest.capabilities,
        vec![
            RUNTIME_CAPABILITY.to_string(),
            SOLARI_CAPABILITY.to_string()
        ]
    );
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == RUNTIME_CAPABILITY
            && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
    }));
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == SOLARI_CAPABILITY
            && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            && status.note.as_deref() == Some(SOLARI_UNAVAILABLE_MESSAGE)
    }));
}

#[test]
fn solari_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("solari dist distribution");

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, "zircon_plugin_solari_dist");
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        "zircon_plugin_solari_runtime_entry_v3"
    );
    assert!(manifest.modules.iter().any(|module| {
        module.kind == zircon_runtime::plugin::PluginModuleKind::Native
            && module.name == "solari.dist"
            && module.crate_name == "zircon_plugin_solari_dist"
            && module.capabilities == RUNTIME_CAPABILITIES
    }));
}
