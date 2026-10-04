// 这里只检查分隔符形态；组合入口还要求首字符为小写字母，
// 子规则测试接受的前导下划线并不表示完整插件 ID 允许它。
pub(super) fn validate_runtime_plugin_feature_token_underscore(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_ends_with_underscore = false;
    let mut previous_was_underscore = false;
    let invalid = value.bytes().any(|byte| {
        if byte == b'_' {
            let invalid = previous_was_underscore;
            segment_ends_with_underscore = true;
            previous_was_underscore = true;
            invalid
        } else {
            segment_ends_with_underscore = false;
            previous_was_underscore = false;
            false
        }
    }) || segment_ends_with_underscore;

    if invalid {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

#[cfg(test)]
#[path = "tests/underscore.rs"]
mod tests;
