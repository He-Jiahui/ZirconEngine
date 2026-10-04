mod coverage;
mod module;
mod modules;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

/// 补充特性通用校验没有携带的包级目标约束：模块声明的每个目标都须由承载包支持。
/// 这检查声明一致性，不负责判断当前项目是否选择或启用了该特性。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_feature_target_coverage(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    modules::validate_runtime_plugin_package_feature_module_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
