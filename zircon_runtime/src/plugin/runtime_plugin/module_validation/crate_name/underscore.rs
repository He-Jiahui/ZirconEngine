/// 与前缀和字符集检查共同维持插件 crate 的命名规范；调用者应使用组合入口。
/// 单独执行此规则不会证明名称非空或具有插件前缀。
pub(super) fn validate_runtime_plugin_module_crate_name_underscore(
    manifest_label: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut ends_with_underscore = false;
    let mut previous_was_underscore = false;
    let invalid = crate_name.bytes().any(|byte| {
        if byte == b'_' {
            let invalid = previous_was_underscore;
            ends_with_underscore = true;
            previous_was_underscore = true;
            invalid
        } else {
            ends_with_underscore = false;
            previous_was_underscore = false;
            false
        }
    }) || ends_with_underscore;

    if invalid {
        diagnostics.push(format!(
            "{manifest_label} module crate_name `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

#[cfg(test)]
#[path = "tests/underscore.rs"]
mod tests;
