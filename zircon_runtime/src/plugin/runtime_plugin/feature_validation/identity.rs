use crate::plugin::PluginFeatureBundleManifest;

use super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
    validate_runtime_plugin_feature_token,
};

pub(super) fn validate_runtime_plugin_feature_identity(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_namespace("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_field("display_name", &feature.display_name, diagnostics);
    validate_runtime_plugin_feature_field("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    validate_runtime_plugin_feature_token("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    let owner_prefix = format!("{}.", feature.owner_plugin_id);
    if !feature.id.starts_with(&owner_prefix) {
        diagnostics.push(format!(
            "runtime plugin feature manifest feature id `{}` must be prefixed by owner_plugin_id `{}`",
            feature.id, feature.owner_plugin_id
        ));
    }
}
