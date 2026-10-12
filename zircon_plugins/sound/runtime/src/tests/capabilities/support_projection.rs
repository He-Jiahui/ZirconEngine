pub(super) fn project_capability_signatures(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<String> {
    feature.capabilities.clone()
}
