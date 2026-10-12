use crate::plugin::PluginPackageManifest;

/// 扩展包必须通过扩展列表声明其提供的特性，不能同时借用普通包的可选列表入口。
/// 发现列表组合错误后保留全部行，后续阶段仍逐行检查并累计诊断。
pub(super) fn validate_feature_extension_package_feature_kind(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if package_manifest.feature_extensions.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest package_kind FeatureExtension must declare at least one feature_extension"
                .to_string(),
        );
    }
    if !package_manifest.optional_features.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest package_kind FeatureExtension must not declare optional_features"
                .to_string(),
        );
    }
}
