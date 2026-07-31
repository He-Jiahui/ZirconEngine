use super::super::uniqueness::validate_runtime_plugin_feature_capability_uniqueness;

pub(super) fn validate_runtime_plugin_feature_capability_row_uniqueness(
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_capability_uniqueness(capability, is_duplicate, diagnostics);
}
