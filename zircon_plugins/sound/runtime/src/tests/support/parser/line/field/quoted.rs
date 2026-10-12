// 仅剥离当前固定清单的首尾引号，调用者不应把它当成完整 TOML 解码器。
pub(in super::super) fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    super::raw::raw_value(line, prefix).and_then(|value| value.strip_suffix('"'))
}
