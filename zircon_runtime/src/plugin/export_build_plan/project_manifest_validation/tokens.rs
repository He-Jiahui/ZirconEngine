use crate::{
    plugin::{ProjectPluginFeatureSelection, ProjectPluginSelection},
    RuntimeTargetMode,
};

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
    let segments = value.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && segments
            .iter()
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
