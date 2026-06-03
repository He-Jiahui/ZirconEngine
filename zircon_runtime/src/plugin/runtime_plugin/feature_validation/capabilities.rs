mod presence;
mod row;
mod rows;
mod uniqueness;

pub(super) fn validate_runtime_plugin_feature_capabilities(
    capabilities: &[String],
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_feature_capability_presence(capabilities, diagnostics);
    rows::validate_runtime_plugin_feature_capability_rows(capabilities, diagnostics);
}
