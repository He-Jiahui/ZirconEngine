use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_capabilities;
use super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
};

pub(super) fn validate_runtime_plugin_feature_module_capabilities(
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_capabilities(
        "runtime plugin feature manifest",
        module,
        Some(validate_runtime_plugin_feature_field),
        validate_runtime_plugin_feature_namespace,
        diagnostics,
    );
}
