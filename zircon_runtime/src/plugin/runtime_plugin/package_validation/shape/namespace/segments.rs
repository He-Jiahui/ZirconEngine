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
mod tests {
    #[test]
    fn package_namespace_validation_streams_segments() {
        let source = include_str!("segments.rs");
        let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&allocating_shape));
    }

    #[test]
    fn package_namespace_validation_preserves_segment_diagnostics() {
        let mut diagnostics = Vec::new();
        super::validate_runtime_plugin_package_namespace_segments(
            "feature id",
            "rendering",
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("at least two dot-separated namespace segments"));

        diagnostics.clear();
        super::validate_runtime_plugin_package_namespace_segments(
            "feature id",
            "rendering..deferred",
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("must contain only lowercase ASCII"));
    }
}
