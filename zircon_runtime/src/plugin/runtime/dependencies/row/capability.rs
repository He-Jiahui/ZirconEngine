use crate::plugin::PluginFeatureDependency;

use super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
};

pub(super) fn validate_runtime_plugin_feature_dependency_capability(
    dependency: &PluginFeatureDependency,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field(
        "dependency capability",
        &dependency.capability,
        diagnostics,
    );
    validate_runtime_plugin_feature_namespace(
        "dependency capability",
        &dependency.capability,
        diagnostics,
    );
}
