use super::*;

#[test]
fn virtual_geometry_registration_contributes_render_feature_descriptor() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == VIRTUAL_GEOMETRY_MODULE_NAME));
    assert_eq!(
        report.extensions.render_features()[0].name,
        VIRTUAL_GEOMETRY_FEATURE_NAME
    );
    assert_eq!(
        report.extensions.virtual_geometry_runtime_providers()[0].provider_id(),
        PLUGIN_ID
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(report.package_manifest.category, "rendering");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    assert!(report
        .package_manifest
        .capabilities
        .contains(&VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY.to_string()));
    assert!(report.package_manifest.modules[0]
        .capabilities
        .contains(&VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY.to_string()));
    let feature = &report.extensions.render_features()[0];
    assert_eq!(
        feature.required_extract_sections,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string()
        ]
    );
    assert_eq!(
        feature.capability_requirements,
        vec![zircon_runtime::graphics::RenderFeatureCapabilityRequirement::VirtualGeometry]
    );
    assert_eq!(
        feature
            .stage_passes
            .iter()
            .map(|pass| pass.pass_name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "virtual-geometry-prepare",
            "virtual-geometry-node-cluster-cull",
            "virtual-geometry-page-feedback",
            "virtual-geometry-visbuffer",
            "virtual-geometry-debug-overlay",
        ]
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 5);
    assert_eq!(report.extensions.runtime_prepare_collectors().len(), 1);
    assert_eq!(
        report.extensions.runtime_prepare_collectors()[0].collector_id(),
        "virtual-geometry.runtime-prepare"
    );
    assert_eq!(
        report
            .extensions
            .render_pass_executors()
            .iter()
            .map(|registration| registration.executor_id().as_str())
            .collect::<Vec<_>>(),
        vec![
            "virtual-geometry.prepare",
            "virtual-geometry.node-cluster-cull",
            "virtual-geometry.page-feedback",
            "virtual-geometry.visbuffer",
            "virtual-geometry.debug-overlay",
        ]
    );
}

#[test]
fn virtual_geometry_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("virtual_geometry distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, VIRTUAL_GEOMETRY_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        VIRTUAL_GEOMETRY_DIST_RUNTIME_ENTRY
    );

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "virtual_geometry.dist")
        .expect("virtual_geometry native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, VIRTUAL_GEOMETRY_DIST_CRATE_NAME);
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
