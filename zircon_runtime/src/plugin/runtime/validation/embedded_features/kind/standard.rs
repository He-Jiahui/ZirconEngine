use crate::plugin::PluginPackageManifest;

/// 普通包通过可选列表声明特性；独立提供者的扩展列表应放在对应类型的包中。
pub(super) fn validate_standard_package_feature_kind(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.feature_extensions.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest Standard package_kind must not declare feature_extensions"
                .to_string(),
        );
    }
}
