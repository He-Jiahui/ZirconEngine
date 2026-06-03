use super::shape::{validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_token};

pub(super) fn validate_runtime_plugin_feature_provider_package_id(
    provider_package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("provider_package_id", provider_package_id, diagnostics);
    validate_runtime_plugin_feature_token("provider_package_id", provider_package_id, diagnostics);
}
