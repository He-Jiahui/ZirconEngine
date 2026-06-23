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
