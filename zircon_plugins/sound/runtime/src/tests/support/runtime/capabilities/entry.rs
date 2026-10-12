pub(in super::super) fn capability_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<String> {
    let mut capabilities = super::projection::project_capability_signatures(feature);
    super::ordering::sort_capability_signatures(&mut capabilities);
    capabilities
}
