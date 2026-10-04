// 固定清单只使用逗号分隔的简单字符串数组；调用者不可把此测试投影当作通用 TOML 解码器。
pub(super) fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}
