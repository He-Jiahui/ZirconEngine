use super::*;

#[test]
fn prefab_runtime_plugin_contributes_component_and_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .components()
        .iter()
        .any(|component| component.type_id == PREFAB_INSTANCE_COMPONENT_TYPE));
    assert_eq!(
        report.package_manifest.asset_importers[0].full_suffixes,
        vec![".prefab.toml".to_string()]
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
            status.capability == "runtime.plugin.prefab_tools"
                && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
        }));
}
