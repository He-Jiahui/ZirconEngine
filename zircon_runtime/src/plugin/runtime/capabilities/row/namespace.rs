use super::super::super::shape::validate_runtime_plugin_feature_namespace;

pub(super) fn validate_runtime_plugin_feature_capability_namespace(
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_namespace("capability", capability, diagnostics);
}
