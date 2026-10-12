// 本层允许单个合法片段，供组合入口单独控制最少分段数；
// 分段内容出错只追加一条诊断，避免同一个非法标识符淹没注册报告。
pub(super) fn validate_runtime_plugin_feature_namespace_segment_tokens(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_is_non_empty = false;
    let invalid = value.bytes().any(|byte| {
        if byte == b'.' {
            let invalid = !segment_is_non_empty;
            segment_is_non_empty = false;
            invalid
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' {
            segment_is_non_empty = true;
            false
        } else {
            true
        }
    }) || !segment_is_non_empty;

    if invalid {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
        ));
    }
}

#[cfg(test)]
#[path = "tests/tokens.rs"]
mod tests;
