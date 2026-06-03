mod charset;
mod start;
mod underscore;

pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_token(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    start::validate_runtime_plugin_feature_token_start(field_name, value, diagnostics);
    charset::validate_runtime_plugin_feature_token_charset(field_name, value, diagnostics);
    underscore::validate_runtime_plugin_feature_token_underscore(field_name, value, diagnostics);
}
