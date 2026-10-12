// 固定清单字段的底层前缀匹配；调用层负责限定当前区段和字段含义。
pub(in super::super) fn raw_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix)
}
