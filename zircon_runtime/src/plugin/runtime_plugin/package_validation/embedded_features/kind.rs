mod feature_extension;
mod standard;

use crate::plugin::{PluginPackageKind, PluginPackageManifest};

/// 根据包类型限制允许的特性列表，区分拥有者的可选特性声明与独立扩展包的提供者声明。
/// 包角色是否允许进入产品目录由目录发布阶段另行判断。
pub(super) fn validate_runtime_plugin_package_feature_kind(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    match package_manifest.package_kind {
        PluginPackageKind::Standard => {
            standard::validate_standard_package_feature_kind(package_manifest, diagnostics)
        }
        PluginPackageKind::FeatureExtension => {
            feature_extension::validate_feature_extension_package_feature_kind(
                package_manifest,
                diagnostics,
            );
        }
    }
}
