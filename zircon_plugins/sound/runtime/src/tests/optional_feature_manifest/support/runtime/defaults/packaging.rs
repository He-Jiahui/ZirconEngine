pub(in super::super) fn default_packaging(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> Vec<zircon_runtime::plugin::ExportPackagingStrategy> {
    feature.default_packaging.clone()
}
