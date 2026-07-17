use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginFeatureSelection, ProjectPluginSelection};

pub(super) fn target_consumes_selection(
    selection: &ProjectPluginSelection,
    target: RuntimeTargetMode,
) -> bool {
    selection.enabled && selection.supports_target(target)
}

pub(super) fn target_consumes_feature(
    feature: &ProjectPluginFeatureSelection,
    target: RuntimeTargetMode,
) -> bool {
    feature.enabled && feature.supports_target(target)
}

pub(super) fn is_lowercase_project_plugin_package_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_project_feature_namespace(value: &str) -> bool {
    value.contains('.')
        && value
            .split('.')
            .all(|segment| !segment.is_empty() && is_lowercase_project_feature_segment(segment))
}

pub(super) fn is_lowercase_project_feature_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_project_runtime_crate(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    #[test]
    fn feature_namespace_validation_does_not_collect_split_segments() {
        let source = include_str!("tokens.rs");
        let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(
            !source.contains(&allocating_shape),
            "feature namespace validation should stream split segments without a temporary Vec"
        );
    }
}
