pub(in super::super) fn bracketed_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    super::raw::raw_value(line, prefix).and_then(|value| value.strip_suffix(']'))
}
