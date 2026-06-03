mod capability;
mod provider;

use crate::plugin::PluginFeatureDependency;

use self::{
    capability::validate_runtime_plugin_feature_dependency_capability,
    provider::validate_runtime_plugin_feature_dependency_provider,
};

pub(super) fn validate_runtime_plugin_feature_dependency_row(
    dependency: &PluginFeatureDependency,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_dependency_provider(dependency, diagnostics);
    validate_runtime_plugin_feature_dependency_capability(dependency, diagnostics);
}
