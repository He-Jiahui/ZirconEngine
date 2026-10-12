use super::super::super::shape::validate_runtime_plugin_feature_field;

pub(super) fn validate_runtime_plugin_feature_capability_field(
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("capability", capability, diagnostics);
}
