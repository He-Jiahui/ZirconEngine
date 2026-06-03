use crate::plugin::{PluginFeatureBundleManifest, PluginPackageKind, PluginPackageManifest};

pub(super) fn runtime_plugin_package_feature_provider_package_id<'a>(
    package_manifest: &'a PluginPackageManifest,
    feature: &'a PluginFeatureBundleManifest,
) -> &'a str {
    if package_manifest.package_kind == PluginPackageKind::FeatureExtension
        || feature.owner_plugin_id.as_str() != package_manifest.id.as_str()
    {
        package_manifest.id.as_str()
    } else {
        feature.owner_plugin_id.as_str()
    }
}
