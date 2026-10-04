use crate::plugin::{PluginFeatureBundleManifest, PluginPackageKind, PluginPackageManifest};

// BUG: [CR-PLUGIN-VALIDATION-0330] 扩展包中同名特性的不同显式提供者会绕过包内重复检查，目录随后都按承载包投影为同一键并覆盖前一行；证据：目录的包特性投影与合并路径。
/// 为嵌入特性的重复诊断借用提供者身份；省略提供者时，承载包与特性所属插件可以不同。
/// 返回值只用于当前校验链，不能据此推断目录发布时的实际提供者或其注册状态。
pub(super) fn runtime_plugin_package_feature_provider_package_id<'a>(
    package_manifest: &'a PluginPackageManifest,
    feature: &'a PluginFeatureBundleManifest,
) -> &'a str {
    if let Some(provider_package_id) = feature.provider_package_id.as_deref() {
        return provider_package_id;
    }
    if package_manifest.package_kind == PluginPackageKind::FeatureExtension
        || feature.owner_plugin_id.as_str() != package_manifest.id.as_str()
    {
        package_manifest.id.as_str()
    } else {
        feature.owner_plugin_id.as_str()
    }
}
