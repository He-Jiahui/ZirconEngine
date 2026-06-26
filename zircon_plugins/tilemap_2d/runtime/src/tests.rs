use super::*;

#[test]
fn tilemap_runtime_plugin_contributes_component_and_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .components()
        .iter()
        .any(|component| component.type_id == TILEMAP_COMPONENT_TYPE));
    assert_eq!(
        report.package_manifest.asset_importers[0].source_extensions,
        vec!["tmx".to_string(), "tsx".to_string(), "json".to_string()]
    );
    assert_eq!(report.package_manifest.category, "authoring");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Beta
    );
    assert!(report
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == "runtime.plugin.tilemap_2d"
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
        }));
}

#[test]
fn tilemap_2d_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("tilemap_2d distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, TILEMAP_2D_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, TILEMAP_2D_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "tilemap_2d.dist")
        .expect("tilemap_2d native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, TILEMAP_2D_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
