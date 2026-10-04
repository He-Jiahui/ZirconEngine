mod capability;
mod provider;

use crate::plugin::PluginFeatureDependency;

use self::{
    capability::validate_runtime_plugin_feature_dependency_capability,
    provider::validate_runtime_plugin_feature_dependency_provider,
};

// 提供者与能力分别校验并累积诊断；前项无效时仍会检查后项。
pub(super) fn validate_runtime_plugin_feature_dependency_row(
    dependency: &PluginFeatureDependency,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_dependency_provider(dependency, diagnostics);
    validate_runtime_plugin_feature_dependency_capability(dependency, diagnostics);
}
