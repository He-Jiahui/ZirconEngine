use super::super::uniqueness::validate_runtime_plugin_feature_capability_uniqueness;

pub(super) fn validate_runtime_plugin_feature_capability_row_uniqueness<'a>(
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_capability_uniqueness(capability, seen, diagnostics);
}
