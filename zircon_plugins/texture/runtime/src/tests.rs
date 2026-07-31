use zircon_runtime::core::CoreRuntime;

use super::*;

#[test]
fn declaration_owns_texture_package_metadata() {
    let descriptor = runtime_plugin_descriptor();
    let manifest = package_manifest();

    assert_eq!(TEXTURE_PLUGIN_DECLARATION.id(), PLUGIN_ID);
    assert_eq!(
        runtime_capabilities(),
        TEXTURE_PLUGIN_DECLARATION.capabilities()
    );
    assert_eq!(descriptor.package_id(), TEXTURE_PLUGIN_DECLARATION.id());
    assert_eq!(descriptor.category(), TEXTURE_PLUGIN_DECLARATION.category());
    assert_eq!(descriptor.maturity(), TEXTURE_PLUGIN_DECLARATION.maturity());
    assert_eq!(
        descriptor.target_modes(),
        TEXTURE_PLUGIN_DECLARATION.target_modes()
    );
    assert_eq!(
        descriptor.capabilities(),
        TEXTURE_PLUGIN_DECLARATION
            .capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        manifest.default_packaging.as_slice(),
        TEXTURE_PLUGIN_DECLARATION.default_packaging()
    );
}

#[test]
fn texture_registration_contributes_runtime_module() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(
        report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == TEXTURE_MODULE_NAME)
    );
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Stable
    );
}

#[test]
fn texture_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("texture distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, TEXTURE_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, TEXTURE_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "texture.dist")
        .expect("texture native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, TEXTURE_DIST_CRATE_NAME);
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

#[test]
fn texture_module_resolves_manager_and_summarizes_texture() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(TEXTURE_MODULE_NAME).unwrap();
    let manager = runtime
        .handle()
        .resolve_manager::<DefaultTextureManager>(TEXTURE_MANAGER_NAME)
        .unwrap();

    let summary = manager.summarize_texture(16, 8, 0);

    assert_eq!(summary.mip_count, 1);
    assert_eq!(summary.texel_count, 128);
}
