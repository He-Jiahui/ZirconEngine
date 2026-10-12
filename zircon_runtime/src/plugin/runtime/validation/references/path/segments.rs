/// 拒绝空路径段和目录穿越段；此函数只处理段规则，参考树前缀与分隔符由调用端另行检查。
pub(super) fn validate_runtime_plugin_package_bevy_reference_path_segments(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_len = 0usize;
    let mut dot_count = 0usize;
    let invalid = reference.bytes().any(|byte| {
        if byte == b'/' {
            let invalid = segment_len == 0
                || (segment_len == 1 && dot_count == 1)
                || (segment_len == 2 && dot_count == 2);
            segment_len = 0;
            dot_count = 0;
            invalid
        } else {
            segment_len += 1;
            if byte == b'.' {
                dot_count += 1;
            }
            false
        }
    }) || segment_len == 0
        || (segment_len == 1 && dot_count == 1)
        || (segment_len == 2 && dot_count == 2);

    if invalid {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must not contain empty, current, or parent path segments"
        ));
    }
}

#[cfg(test)]
#[path = "tests/segments.rs"]
mod tests;
