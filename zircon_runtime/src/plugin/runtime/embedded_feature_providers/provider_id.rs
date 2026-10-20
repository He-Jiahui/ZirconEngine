use super::package_validation::EmbeddedFeatureKind;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageKind, PluginPackageManifest};

// BUG: [CR-PLUGIN-VALIDATION-0330] 扩展包中同名特性的不同显式提供者会绕过包内重复检查，目录随后都按承载包投影为同一键并覆盖前一行；证据：目录的包特性投影与合并路径。
/// 为重复索引、诊断和目录发布借用同一提供者身份；扩展行及跨 owner 行使用承载包。
/// 普通 owner 的可选特性保留显式提供者；此身份不证明提供者实际注册。
pub(super) fn runtime_plugin_package_feature_provider_package_id<'a>(
    package_manifest: &'a PluginPackageManifest,
    feature: &'a PluginFeatureBundleManifest,
    kind: EmbeddedFeatureKind,
) -> &'a str {
    if kind == EmbeddedFeatureKind::Extension
        || package_manifest.package_kind == PluginPackageKind::FeatureExtension
        || feature.owner_plugin_id.as_str() != package_manifest.id.as_str()
    {
        package_manifest.id.as_str()
    } else {
        feature
            .provider_package_id
            .as_deref()
            .unwrap_or(feature.owner_plugin_id.as_str())
    }
}
