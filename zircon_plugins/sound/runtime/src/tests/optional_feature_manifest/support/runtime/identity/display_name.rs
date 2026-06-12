pub(in super::super) fn feature_display_name(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> String {
    feature.display_name.clone()
}
