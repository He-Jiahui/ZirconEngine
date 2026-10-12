// 仅适配当前清单的简单逗号分隔字符串数组；这里不是通用 TOML 字符串解析器。
pub(in super::super) fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}
