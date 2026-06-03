mod segments;

pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_namespace(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    segments::validate_runtime_plugin_feature_namespace_segments(field_name, value, diagnostics);
}
