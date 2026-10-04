use super::*;

#[test]
fn rendering_descriptor_declares_fifteen_owner_features() {
    let descriptor = runtime_plugin_descriptor();

    assert_eq!(descriptor.category(), "rendering");
    assert_eq!(
        descriptor.maturity(),
        zircon_runtime::plugin::PluginMaturity::Stable
    );
    assert_eq!(descriptor.optional_features().len(), 15);
    assert!(descriptor
        .optional_features()
        .iter()
        .any(|feature| feature.id == "rendering.ssao" && !feature.enabled_by_default));
    assert!(descriptor
        .optional_features()
        .iter()
        .any(|feature| feature.id == "rendering.contact_shadow" && !feature.enabled_by_default));
    assert!(descriptor
        .optional_features()
        .iter()
        .any(|feature| feature.id == "rendering.oit" && !feature.enabled_by_default));
    assert!(descriptor
        .optional_features()
        .iter()
        .any(|feature| { feature.id == "rendering.light_cookies" && !feature.enabled_by_default }));
    assert!(descriptor.optional_features().iter().any(|feature| {
        feature.id == "rendering.irradiance_volumes" && !feature.enabled_by_default
    }));
    assert!(descriptor.optional_features().iter().any(|feature| {
        feature.id == "rendering.planar_reflections" && !feature.enabled_by_default
    }));
    assert!(descriptor.optional_features().iter().any(|feature| {
        feature.id == "rendering.subsurface_scattering" && !feature.enabled_by_default
    }));
    assert!(descriptor
        .optional_features()
        .iter()
        .any(|feature| feature.id == "rendering.volumetric_fog" && !feature.enabled_by_default));
    assert_eq!(
        descriptor
            .optional_features()
            .iter()
            .filter(|feature| feature.enabled_by_default)
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
        ]
    );
}

#[test]
fn rendering_feature_manifests_declare_editor_capabilities() {
    for feature_kind in RENDERING_FEATURES {
        let manifest = feature_manifest(*feature_kind);
        let editor_capability = feature_kind.editor_capability();
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
            .expect("rendering feature editor module");

        assert!(
            editor_module.capabilities.contains(&editor_capability),
            "{} editor module should project {editor_capability}",
            manifest.id
        );
    }
}

#[test]
fn vfx_graph_requires_particles_and_shader_graph() {
    let manifest = feature_manifest(RenderingFeatureKind::VfxGraph);

    assert!(manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "particles" && dependency.capability == "runtime.plugin.particles"
    }));
    assert!(manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == PLUGIN_ID
            && dependency.capability == "runtime.feature.rendering.shader_graph"
    }));
}

#[test]
fn rendering_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("rendering distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, RENDERING_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, RENDERING_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "rendering.dist")
        .expect("rendering native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, RENDERING_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
