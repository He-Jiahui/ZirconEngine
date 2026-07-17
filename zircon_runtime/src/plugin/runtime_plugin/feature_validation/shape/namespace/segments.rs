mod count;
mod tokens;

use self::{
    count::validate_runtime_plugin_feature_namespace_segment_count,
    tokens::validate_runtime_plugin_feature_namespace_segment_tokens,
};

pub(super) fn validate_runtime_plugin_feature_namespace_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !validate_runtime_plugin_feature_namespace_segment_count(field_name, value, diagnostics) {
        return;
    }
    validate_runtime_plugin_feature_namespace_segment_tokens(field_name, value, diagnostics);
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_feature_namespace_validation_streams_segments() {
        let source = include_str!("segments.rs");
        let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&allocating_shape));
    }
}
