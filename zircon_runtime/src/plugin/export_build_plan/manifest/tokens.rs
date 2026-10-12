//! 导出清单的 ASCII token 基础判定；身份、crate 与 provider 校验共享此规则。
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

/// owner.feature 的共享字节规则；身份诊断据此拒绝空段或非 ASCII 标识。
pub(super) fn is_lowercase_project_feature_namespace(value: &str) -> bool {
    let mut saw_separator = false;
    let mut segment_is_non_empty = false;
    for byte in value.bytes() {
        if byte == b'.' {
            if !segment_is_non_empty {
                return false;
            }
            saw_separator = true;
            segment_is_non_empty = false;
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' {
            segment_is_non_empty = true;
        } else {
            return false;
        }
    }
    saw_separator && segment_is_non_empty
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
#[path = "tests/tokens.rs"]
mod tests;
