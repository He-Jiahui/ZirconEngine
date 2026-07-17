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
    if !feature_id_has_owner(&feature.owner_plugin_id, &feature.id) {
        diagnostics.push(format!(
            "runtime plugin feature manifest feature id `{}` must be prefixed by owner_plugin_id `{}`",
            feature.id, feature.owner_plugin_id
        ));
    }
}

fn feature_id_has_owner(owner_plugin_id: &str, feature_id: &str) -> bool {
    feature_id
        .strip_prefix(owner_plugin_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_feature_owner_prefix_check_does_not_format_a_string() {
        let source = include_str!("identity.rs");
        let formatted_prefix = ["format!(\"{}", ".\", feature.owner_plugin_id)"].concat();
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn runtime_feature_owner_matching_preserves_the_dot_boundary() {
        assert!(super::feature_id_has_owner(
            "rendering",
            "rendering.deferred"
        ));
        assert!(!super::feature_id_has_owner("render", "rendering.deferred"));
    }
}
