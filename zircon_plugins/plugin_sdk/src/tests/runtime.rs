use super::*;

#[test]
fn runtime_declaration_projects_descriptor_and_manifest_from_one_source() {
    let declaration = RuntimePluginDeclaration::new(
        "navigation",
        "Navigation",
        RuntimePluginId::Navigation,
        "zircon_plugin_navigation_runtime",
    )
    .with_category("runtime")
    .with_maturity(PluginMaturity::Beta)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
    ])
    .with_capability("runtime.plugin.navigation")
    .with_system_anchors(["navigation.runtime.tick"]);

    let descriptor = declaration.descriptor();
    let manifest = declaration.package_manifest();

    assert_eq!(descriptor.package_id(), "navigation");
    assert_eq!(
        descriptor.capabilities(),
        ["runtime.plugin.navigation".to_string()]
    );
    assert_eq!(manifest.id, descriptor.package_id());
    assert_eq!(manifest.category, descriptor.category());
    assert_eq!(manifest.maturity, descriptor.maturity());
    assert_eq!(manifest.supported_targets, descriptor.target_modes());
    assert_eq!(manifest.capabilities, descriptor.capabilities());
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.name == "navigation.runtime"
            && module.crate_name == "zircon_plugin_navigation_runtime"
            && module.system_anchors == ["navigation.runtime.tick".to_string()]));
}
