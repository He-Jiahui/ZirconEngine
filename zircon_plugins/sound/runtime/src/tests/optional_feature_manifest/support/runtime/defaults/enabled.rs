pub(super) fn enabled_by_default(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> bool {
    feature.enabled_by_default
}
