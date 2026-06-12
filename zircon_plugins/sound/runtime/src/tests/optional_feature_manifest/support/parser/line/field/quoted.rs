pub(in super::super) fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    super::raw::raw_value(line, prefix).and_then(|value| value.strip_suffix('"'))
}
