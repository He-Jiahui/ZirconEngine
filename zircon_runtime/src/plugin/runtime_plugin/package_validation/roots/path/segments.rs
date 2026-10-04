// 禁止空段及导航段，使清单根路径在后续拼接时保持声明的目录层级；此处不解析磁盘符号链接。
pub(super) fn validate_runtime_plugin_package_root_segments(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_len = 0usize;
    let mut dot_count = 0usize;
    let invalid = root.bytes().any(|byte| {
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
            "runtime plugin package manifest {field_name} root `{root}` must not contain empty, current, or parent path segments"
        ));
    }
}

#[cfg(test)]
#[path = "tests/segments.rs"]
mod tests;
