// 仅识别固定清单中的单行数组字段，完整值域由上层列表转换器解释。
pub(in super::super) fn bracketed_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    super::raw::raw_value(line, prefix).and_then(|value| value.strip_suffix(']'))
}
