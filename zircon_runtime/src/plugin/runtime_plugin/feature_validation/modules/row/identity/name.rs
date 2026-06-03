use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use super::super::super::super::super::module_validation::validate_runtime_plugin_module_name;
use super::super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
};

pub(super) fn validate_runtime_plugin_feature_module_name<'a>(
    feature: &PluginFeatureBundleManifest,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name(
        "runtime plugin feature manifest",
        "feature id",
        &feature.id,
        module,
        seen_names,
        validate_runtime_plugin_feature_field,
        validate_runtime_plugin_feature_namespace,
        diagnostics,
    );
}
