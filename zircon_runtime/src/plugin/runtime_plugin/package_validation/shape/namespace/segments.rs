mod count;
mod tokens;

use self::{
    count::validate_runtime_plugin_package_namespace_segment_count,
    tokens::validate_runtime_plugin_package_namespace_segment_tokens,
};

pub(super) fn validate_runtime_plugin_package_namespace_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !validate_runtime_plugin_package_namespace_segment_count(field_name, value, diagnostics) {
        return;
    }
    validate_runtime_plugin_package_namespace_segment_tokens(field_name, value, diagnostics);
}

#[cfg(test)]
#[path = "tests/segments.rs"]
mod tests;
