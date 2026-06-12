pub(in super::super) fn feature_id(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> String {
    feature.id.clone()
}
