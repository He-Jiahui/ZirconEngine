#[path = "../embedded_feature_providers/uniqueness.rs"]
mod uniqueness;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use self::uniqueness::validate_runtime_plugin_package_feature_provider_uniqueness;
use super::EmbeddedFeatureKind;
use crate::plugin::runtime_plugin::embedded_feature_provider_identity::runtime_plugin_package_feature_provider_package_id;

/// 把共享投影中同一特性与提供者组合的重复结果转为包诊断，而非要求特性名全包唯一。
/// 重复标志须来自当前特性在该清单中的类别及行号；此入口不检查提供者是否实际注册。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_feature_provider(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    kind: EmbeddedFeatureKind,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    let provider_package_id =
        runtime_plugin_package_feature_provider_package_id(package_manifest, feature, kind);
    validate_runtime_plugin_package_feature_provider_uniqueness(
        field_name,
        &feature.id,
        provider_package_id,
        is_duplicate,
        diagnostics,
    );
}
