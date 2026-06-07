pub(super) fn feature_owner_plugin_id(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> String {
    feature.owner_plugin_id.clone()
}
