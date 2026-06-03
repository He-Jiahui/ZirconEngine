use crate::plugin::PluginFeatureDependency;

use super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_token,
};

pub(super) fn validate_runtime_plugin_feature_dependency_provider(
    dependency: &PluginFeatureDependency,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field(
        "dependency plugin_id",
        &dependency.plugin_id,
        diagnostics,
    );
    validate_runtime_plugin_feature_token(
        "dependency plugin_id",
        &dependency.plugin_id,
        diagnostics,
    );
}
